"use client";
import { useState, useEffect, useCallback, useRef } from "react";
import { MdArrowBack, MdAdd, MdDelete, MdPictureAsPdf, MdDescription, MdVisibility, MdEdit, MdAssignmentTurnedIn, MdLibraryAdd, MdShuffle } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import RichEditor from "@/components/RichEditor";
import "katex/dist/katex.min.css";
import type { Materia, Prova, Questao, QuestaoInput, OpcaoQuestao, TipoQuestao, Turma, ToastState, BancoQuestao } from "@/types";

const TIPOS: TipoQuestao[] = ["dissertativa", "multipla_escolha", "verdadeiro_falso", "completar_lacunas", "associacao", "ordenar"];
const TIPO_LABELS: Record<TipoQuestao, string> = {
  dissertativa: "Dissertativa",
  multipla_escolha: "Múltipla Escolha",
  verdadeiro_falso: "Verdadeiro ou Falso",
  completar_lacunas: "Completar Lacunas",
  associacao: "Associação",
  ordenar: "Ordenar/Sequenciar",
};

interface ProvaForm {
  titulo: string; descricao: string; materia_id: string;
  data: string; rodape: string; margens: string; valor_total: string;
  escola_override: string; cidade_override: string; turma_id: string;
  is_recuperacao: boolean; qr_gabarito: boolean;
  duas_colunas: boolean; paisagem: boolean;
  [k: string]: unknown;
}

interface Props {
  provaId: number | null;
  materias: Materia[];
  turmas: Turma[];
  onClose: () => void;
  onNotify: (message: string, type?: ToastState["type"]) => void;
}

const EMPTY_PROVA: ProvaForm = { titulo: "", descricao: "", materia_id: "", data: "", rodape: "", margens: "normal", valor_total: "10", escola_override: "", cidade_override: "", turma_id: "", is_recuperacao: false, qr_gabarito: false, duas_colunas: false, paisagem: false };
const newQuestao = (): QuestaoInput => ({ enunciado: "", tipo: "dissertativa", opcoes: [], valor: 0, linhas_resposta: 3, tags: "", dificuldade: "médio", tempId: Date.now() + Math.random() });

function somaQuestoes(questoes: QuestaoInput[]) {
  return questoes.reduce((acc, q) => acc + (Number(q.valor) || 0), 0);
}

// Renders stored Tiptap HTML with KaTeX math re-rendered client-side
function EnunciadoPreview({ html }: { html: string }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const el = ref.current;

    // 1. Render Tiptap math nodes: <span data-type="inline-math" data-latex="...">
    //    and <div data-type="block-math" data-latex="...">
    el.querySelectorAll<HTMLElement>("[data-type='inline-math'], [data-type='block-math']").forEach((node) => {
      const latex = node.getAttribute("data-latex") ?? "";
      const display = node.getAttribute("data-type") === "block-math";
      try {
        import("katex").then(({ default: katex }) => {
          node.innerHTML = katex.renderToString(latex, { displayMode: display, throwOnError: false });
        });
      } catch { /* ignore */ }
    });

    // 2. Also handle legacy $...$ plain-text math (old DB entries)
    import("katex/contrib/auto-render").then((mod) => {
      mod.default(el, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "$", right: "$", display: false },
        ],
        throwOnError: false,
      });
    });
  }, [html]);

  return <div ref={ref} className="preview-enunciado" dangerouslySetInnerHTML={{ __html: html }} />;
}

function renderBlanks(text: string) {
  const parts = text.split(/_{2,}/g);
  if (parts.length === 1) return <>{text}</>;
  return (
    <>
      {parts.map((part, i) => (
        <span key={i}>
          {part}
          {i < parts.length - 1 && (
            <span className="inline-block border-b-2 border-gray-700 w-16 mx-0.5 align-bottom" />
          )}
        </span>
      ))}
    </>
  );
}

function PreviewProva({ form, questoes, materias }: { form: ProvaForm; questoes: QuestaoInput[]; materias: Materia[] }) {
  const materia = materias.find((m) => String(m.id) === form.materia_id);
  return (
    <div className="bg-white text-black p-8 rounded shadow-lg font-serif text-sm leading-relaxed max-w-2xl mx-auto">
      <div className="text-center mb-4 border-b pb-3">
        {materia && <p className="text-xs text-gray-500">{materia.nome}{materia.professor_nome ? ` — Prof(a): ${materia.professor_nome}` : ""}</p>}
        {form.titulo && <h2 className="text-lg font-bold mt-1">{form.titulo}</h2>}
        {form.data && <p className="text-xs text-gray-500">{form.data}</p>}
        <div className="mt-2 text-xs text-left space-y-1">
          <p>Aluno(a): _____________________________________ Nota: ______</p>
          <p>Série/Ano: _________________ Turno: _________________</p>
        </div>
      </div>
      {form.descricao && <p className="mb-4 italic text-xs">{form.descricao}</p>}
      <ol className="space-y-4">
        {questoes.map((q, i) => (
          <li key={q.id ?? q.tempId ?? i}>
            <div className="font-semibold mb-1">Questão {i + 1} ({q.valor} pt)</div>
            <div className="mb-1">{q.tipo === "completar_lacunas" ? renderBlanks(q.enunciado) : <EnunciadoPreview html={q.enunciado} />}</div>
            {q.tipo === "multipla_escolha" && (
              <ul className="mt-1 ml-4 space-y-1">
                {q.opcoes.map((o, j) => (
                  <li key={j}>({String.fromCharCode(97 + j)}) {o.texto}</li>
                ))}
              </ul>
            )}
            {q.tipo === "verdadeiro_falso" && (
              <ul className="mt-1 ml-4 space-y-1">
                {q.opcoes.map((o, j) => (
                  <li key={j}>{String.fromCharCode(97 + j)}) (   ) {o.texto}</li>
                ))}
              </ul>
            )}
            {q.tipo === "completar_lacunas" && q.opcoes.length > 0 && (
              <div className="ml-4 mt-2 flex flex-wrap items-center gap-1">
                <span className="text-xs text-gray-500">Banco de palavras:</span>
                {q.opcoes.filter((o) => o.texto).map((o, j) => (
                  <span key={j} className="px-2 py-0.5 bg-gray-100 border border-gray-300 rounded-full text-xs">{o.texto}</span>
                ))}
              </div>
            )}
            {q.tipo === "associacao" && (
              <div className="ml-4 mt-2">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    {q.opcoes.map((o, j) => (
                      <p key={j}>{j + 1}) {o.texto}</p>
                    ))}
                  </div>
                  <div>
                    {q.opcoes.map((o, j) => (
                      <p key={j}>{String.fromCharCode(65 + j)}) (   ) {o.par}</p>
                    ))}
                  </div>
                </div>
              </div>
            )}
            {q.tipo === "ordenar" && (
              <ul className="mt-1 ml-4 space-y-1">
                {q.opcoes.map((o, j) => (
                  <li key={j}>(   ) {o.texto}</li>
                ))}
              </ul>
            )}
            {q.tipo === "dissertativa" && (
              <div className="ml-4 mt-1 space-y-1">
                {Array.from({ length: q.linhas_resposta }).map((_, j) => (
                  <p key={j} className="border-b border-gray-300">&nbsp;</p>
                ))}
              </div>
            )}
          </li>
        ))}
      </ol>
      {form.rodape && <p className="mt-6 pt-3 border-t text-xs text-gray-500 text-center">{form.rodape}</p>}
    </div>
  );
}

export default function ProvaEditor({ provaId, materias, turmas, onClose, onNotify }: Props) {
  const [form, setForm] = useState<ProvaForm>(EMPTY_PROVA);
  const [questoes, setQuestoes] = useState<QuestaoInput[]>([]);
  const [saving, setSaving] = useState(false);
  const [preview, setPreview] = useState(false);
  const [bancoModal, setBancoModal] = useState(false);
  const [bancoQuestoes, setBancoQuestoes] = useState<BancoQuestao[]>([]);
  const [versaoModal, setVersaoModal] = useState(false);
  const [versao, setVersao] = useState("A");

  const load = useCallback(async () => {
    if (!provaId) return;
    const prova = await invokeCmd<Prova>("get_prova", { id: provaId });
    setForm({
      titulo: prova.titulo, descricao: prova.descricao,
      materia_id: prova.materia_id ? String(prova.materia_id) : "",
      data: prova.data, rodape: prova.rodape, margens: prova.margens,
      valor_total: String(prova.valor_total),
      escola_override: prova.escola_override ?? "",
      cidade_override: prova.cidade_override ?? "",
      turma_id: prova.turma_id ? String(prova.turma_id) : "",
      is_recuperacao: prova.is_recuperacao ?? false,
      qr_gabarito: prova.qr_gabarito ?? false,
      duas_colunas: prova.duas_colunas ?? false,
      paisagem: prova.paisagem ?? false,
    });
    const qs = await invokeCmd<Questao[]>("list_questoes", { provaId });
    setQuestoes(qs.map((q) => ({
      id: q.id, enunciado: q.enunciado, tipo: q.tipo,
      opcoes: q.opcoes, valor: q.valor, linhas_resposta: q.linhas_resposta,
      tags: q.tags ?? "", dificuldade: q.dificuldade ?? "médio",
    })));
  }, [provaId]);

  useEffect(() => { load(); }, [load]);

  function distribuirPontosIgual() {
    if (questoes.length === 0) return;
    const vt = parseFloat(form.valor_total) || 10;
    const porQuestao = Math.round((vt / questoes.length) * 100) / 100;
    setQuestoes(questoes.map((q) => ({ ...q, valor: porQuestao })));
  }

  function addQuestao() {
    setQuestoes([...questoes, newQuestao()]);
  }

  function removeQuestao(idx: number) {
    setQuestoes(questoes.filter((_, i) => i !== idx));
  }

  function updateQuestao(idx: number, field: keyof QuestaoInput, value: unknown) {
    setQuestoes(questoes.map((q, i) => i === idx ? { ...q, [field]: value } : q));
  }

  function addOpcao(idx: number) {
    const q = questoes[idx];
    updateQuestao(idx, "opcoes", [...q.opcoes, { texto: "", correta: false }]);
  }

  function updateOpcao(qIdx: number, oIdx: number, field: keyof OpcaoQuestao | "par", value: string | boolean) {
    const novas = questoes[qIdx].opcoes.map((o, i) => i === oIdx ? { ...o, [field]: value } : o);
    updateQuestao(qIdx, "opcoes", novas);
  }

  function removeOpcao(qIdx: number, oIdx: number) {
    updateQuestao(qIdx, "opcoes", questoes[qIdx].opcoes.filter((_, i) => i !== oIdx));
  }

  async function handleSave() {
    for (let i = 0; i < questoes.length; i++) {
      const q = questoes[i];
      if (q.tipo === "multipla_escolha" && !q.opcoes.some((o) => o.correta)) {
        onNotify(`Questão ${i + 1}: marque ao menos uma opção correta`, "error");
        return;
      }
      if (q.tipo === "verdadeiro_falso" && q.opcoes.some((o) => !o.texto.trim())) {
        onNotify(`Questão ${i + 1}: preencha o texto de todas as afirmações`, "error");
        return;
      }
    }
    if (form.data && form.data < new Date().toISOString().slice(0, 10)) {
      if (!window.confirm("A data da prova está no passado. Deseja salvar assim?")) return;
    }
    setSaving(true);
    try {
      const payload = { titulo: form.titulo, descricao: form.descricao, materiaId: form.materia_id ? Number(form.materia_id) : null, data: form.data, rodape: form.rodape, margens: form.margens, valorTotal: parseFloat(form.valor_total) || 10, escolaOverride: form.escola_override, cidadeOverride: form.cidade_override, turmaId: form.turma_id ? Number(form.turma_id) : null, isRecuperacao: form.is_recuperacao, qrGabarito: form.qr_gabarito, duasColunas: form.duas_colunas, paisagem: form.paisagem };
      let id = provaId;
      if (!id) {
        id = await invokeCmd<number>("create_prova", payload);
      } else {
        await invokeCmd("update_prova", { id, ...payload });
      }
      await invokeCmd("replace_questoes", { provaId: id, questoes });
      onNotify("Prova salva.");
      onClose();
    } catch (e) {
      onNotify(`Erro ao salvar: ${e}`, "error");
    } finally {
      setSaving(false);
    }
  }

  async function handleExport(format: "pdf" | "word") {
    if (!provaId) { onNotify("Salve a prova antes de exportar.", "warning"); return; }
    const ext = format === "pdf" ? "pdf" : "docx";
    const safeName = (form.titulo || "prova").replace(/[^a-zA-Z0-9\s]/g, "").trim().replace(/\s+/g, "_");
    const filePath = await save({
      defaultPath: `${safeName}.${ext}`,
      filters: [{ name: format === "pdf" ? "PDF" : "Word", extensions: [ext] }],
    });
    if (!filePath) return;
    const cmd = format === "pdf" ? "export_prova_pdf" : "export_prova_word";
    try {
      await invokeCmd(cmd, { id: provaId, path: filePath });
      onNotify("Arquivo exportado com sucesso.");
    } catch (e) {
      onNotify(`Erro ao exportar: ${e}`, "error");
    }
  }

  async function handleExportGabarito() {
    if (!provaId) { onNotify("Salve a prova antes de exportar.", "warning"); return; }
    const safeName = (form.titulo || "prova").replace(/[^a-zA-Z0-9\s]/g, "").trim().replace(/\s+/g, "_");
    const filePath = await save({
      defaultPath: `${safeName}_gabarito.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!filePath) return;
    try {
      await invokeCmd("export_gabarito_pdf", { id: provaId, path: filePath });
      onNotify("Gabarito exportado com sucesso.");
    } catch (e) {
      onNotify(`Erro ao exportar gabarito: ${e}`, "error");
    }
  }

  async function handleExportVersao() {
    if (!provaId) { onNotify("Salve a prova antes de exportar.", "warning"); return; }
    const safeName = (form.titulo || "prova").replace(/[^a-zA-Z0-9\s]/g, "").trim().replace(/\s+/g, "_");
    const filePath = await save({
      defaultPath: `${safeName}_versao_${versao}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!filePath) return;
    setVersaoModal(false);
    try {
      await invokeCmd("export_prova_pdf_embaralhada", { id: provaId, path: filePath, versao });
      onNotify("Versão embaralhada exportada com sucesso.");
    } catch (e) {
      onNotify(`Erro ao exportar: ${e}`, "error");
    }
  }

  async function abrirBanco() {
    setBancoQuestoes(await invokeCmd<BancoQuestao[]>("list_banco_questoes"));
    setBancoModal(true);
  }

  async function importarDoBanco(bancoId: number) {
    if (!provaId) { onNotify("Salve a prova antes de importar.", "warning"); return; }
    try {
      await invokeCmd("import_from_banco", { bancoId, provaId });
      onNotify("Questão importada.");
      setBancoModal(false);
      load();
    } catch (e) {
      onNotify(`Erro ao importar: ${e}`, "error");
    }
  }

  const soma = somaQuestoes(questoes);
  const valorTotal = parseFloat(form.valor_total) || 10;
  const somaOk = Math.abs(soma - valorTotal) < 0.01;

  return (
    <div>
      <div className="flex items-center gap-3 mb-6 flex-wrap">
        <button className="btn btn-ghost" onClick={onClose}><MdArrowBack size={20} /></button>
        <h1 className="text-2xl font-bold flex-1">{provaId ? "Editar Prova" : "Nova Prova"}</h1>
        <button className="btn btn-sm btn-ghost gap-1" onClick={() => setPreview(!preview)}>
          {preview ? <MdEdit size={18} /> : <MdVisibility size={18} />}
          {preview ? "Editar" : "Pré-visualizar"}
        </button>
        {provaId && (
          <>
            <button className="btn btn-sm btn-outline gap-1" onClick={() => handleExport("pdf")}>
              <MdPictureAsPdf size={18} /> Exportar PDF
            </button>
            <button className="btn btn-sm btn-outline gap-1" onClick={() => setVersaoModal(true)}>
              <MdShuffle size={18} /> Exportar Versão
            </button>
            <button className="btn btn-sm btn-outline gap-1" onClick={handleExportGabarito}>
              <MdAssignmentTurnedIn size={18} /> Gabarito PDF
            </button>
            <button className="btn btn-sm btn-outline gap-1" onClick={() => handleExport("word")}>
              <MdDescription size={18} /> Exportar Word
            </button>
          </>
        )}
      </div>

      {preview ? (
        <PreviewProva form={form} questoes={questoes} materias={materias} />
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Título</legend>
              <input className="input w-full" value={form.titulo} onChange={(e) => setForm({ ...form, titulo: e.target.value })} required />
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Matéria</legend>
              <select className="select w-full" value={form.materia_id} onChange={(e) => setForm({ ...form, materia_id: e.target.value })}>
                <option value="">Selecione</option>
                {materias.map((m) => <option key={m.id} value={m.id}>{m.nome}</option>)}
              </select>
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Turma</legend>
              <select className="select w-full" value={form.turma_id} onChange={(e) => setForm({ ...form, turma_id: e.target.value })}>
                <option value="">Todas as turmas</option>
                {turmas.map((t) => <option key={t.id} value={t.id}>{t.nome}</option>)}
              </select>
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Data</legend>
              <input type="date" className="input w-full" value={form.data} onChange={(e) => setForm({ ...form, data: e.target.value })} />
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Valor Total da Prova</legend>
              <input type="number" step="0.5" min="0" className="input w-full" value={form.valor_total} onChange={(e) => setForm({ ...form, valor_total: e.target.value })} />
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Rodapé</legend>
              <input className="input w-full" value={form.rodape} onChange={(e) => setForm({ ...form, rodape: e.target.value })} placeholder="Ex: Boa sorte!" />
            </fieldset>
            <fieldset className="fieldset md:col-span-2">
              <legend className="fieldset-legend">Instruções / Descrição</legend>
              <input className="input w-full" value={form.descricao} onChange={(e) => setForm({ ...form, descricao: e.target.value })} placeholder="Instruções gerais para os alunos" />
            </fieldset>
          </div>

          <details className="mb-4">
            <summary className="cursor-pointer text-sm font-medium opacity-70 hover:opacity-100">Cabeçalho personalizado</summary>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-3">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Escola (sobrepõe configuração)</legend>
                <input className="input w-full" value={form.escola_override} onChange={(e) => setForm({ ...form, escola_override: e.target.value })} placeholder="Deixe vazio para usar a configuração global" />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Cidade (sobrepõe configuração)</legend>
                <input className="input w-full" value={form.cidade_override} onChange={(e) => setForm({ ...form, cidade_override: e.target.value })} placeholder="Deixe vazio para usar a configuração global" />
              </fieldset>
            </div>
          </details>

          <div className="flex flex-wrap gap-6 mb-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" className="checkbox" checked={form.is_recuperacao} onChange={(e) => setForm({ ...form, is_recuperacao: e.target.checked })} />
              <span className="text-sm">É prova de recuperação / segunda chamada</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" className="checkbox" checked={form.duas_colunas} onChange={(e) => setForm({ ...form, duas_colunas: e.target.checked })} />
              <span className="text-sm">Layout duas colunas (questões objetivas)</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" className="checkbox" checked={form.paisagem} onChange={(e) => setForm({ ...form, paisagem: e.target.checked })} />
              <span className="text-sm">Orientação paisagem</span>
            </label>
          </div>

          <div className="flex items-center justify-between mb-3 flex-wrap gap-2">
            <div className="flex items-center gap-3">
              <h2 className="text-xl font-semibold">Questões</h2>
              <span className={`badge ${somaOk ? "badge-success" : "badge-error"}`}>
                {soma.toFixed(2)} / {valorTotal} pt
              </span>
            </div>
            <div className="flex gap-2">
              <button className="btn btn-sm btn-ghost" onClick={distribuirPontosIgual} disabled={questoes.length === 0}>
                Distribuir pontos igualmente
              </button>
              <button className="btn btn-sm btn-outline" onClick={abrirBanco}><MdLibraryAdd /> Importar do Banco</button>
              <button className="btn btn-sm btn-outline" onClick={addQuestao}><MdAdd /> Questão</button>
            </div>
          </div>

          {!somaOk && questoes.length > 0 && (
            <div className="alert alert-warning mb-4 text-sm">
              A soma dos pontos das questões ({soma.toFixed(2)}) não corresponde ao valor total da prova ({valorTotal}).
            </div>
          )}

          <div className="flex flex-col gap-4">
            {questoes.map((q, idx) => (
              <div key={q.id ?? q.tempId ?? idx} className="card bg-base-200 shadow">
                <div className="card-body gap-3">
                  <div className="flex items-center justify-between">
                    <span className="font-medium">Questão {idx + 1}</span>
                    <button className="btn btn-xs btn-ghost text-error" onClick={() => removeQuestao(idx)}><MdDelete /></button>
                  </div>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                    <fieldset className="fieldset col-span-2 md:col-span-2">
                      <legend className="fieldset-legend">Tipo</legend>
                      <select className="select w-full" value={q.tipo} onChange={(e) => updateQuestao(idx, "tipo", e.target.value as TipoQuestao)}>
                        {TIPOS.map((t) => <option key={t} value={t}>{TIPO_LABELS[t]}</option>)}
                      </select>
                    </fieldset>
                    <fieldset className="fieldset">
                      <legend className="fieldset-legend">Pontos</legend>
                      <input type="number" step="0.5" min="0" className="input w-full" value={q.valor} onChange={(e) => updateQuestao(idx, "valor", parseFloat(e.target.value) || 0)} />
                    </fieldset>
                  </div>
                  <fieldset className="fieldset">
                    <legend className="fieldset-legend">Enunciado</legend>
                    <RichEditor
                      value={q.enunciado}
                      onChange={(html) => updateQuestao(idx, "enunciado", html)}
                      placeholder={q.tipo === "completar_lacunas" ? 'Ex: "O Brasil foi descoberto em ___ por ___"' : "Digite o enunciado da questão"}
                      minHeight={90}
                    />
                  </fieldset>

                  {q.tipo === "dissertativa" && (
                    <fieldset className="fieldset">
                      <legend className="fieldset-legend">Linhas para resposta</legend>
                      <input type="number" min="1" max="20" className="input w-32" value={q.linhas_resposta} onChange={(e) => updateQuestao(idx, "linhas_resposta", parseInt(e.target.value) || 3)} />
                    </fieldset>
                  )}

                  {q.tipo === "verdadeiro_falso" && (
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <div>
                          <span className="text-sm font-medium">Afirmações</span>
                          <p className="text-xs text-base-content/50">O aluno escreve V ou F em cada item</p>
                        </div>
                        <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}><MdAdd /> Afirmação</button>
                      </div>
                      {q.opcoes.map((o, oIdx) => (
                        <div key={oIdx} className="flex gap-2 mb-2 items-center">
                          <span className="text-sm w-6 text-base-content/60">{String.fromCharCode(97 + oIdx)})</span>
                          <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Afirmação ${oIdx + 1}`} />
                          <label className="flex items-center gap-1 text-xs cursor-pointer whitespace-nowrap">
                            <span className="text-base-content/50">Gabarito:</span>
                            <select className="select select-xs" value={o.correta ? "V" : "F"} onChange={(e) => updateOpcao(idx, oIdx, "correta", e.target.value === "V")}>
                              <option value="V">Verdadeiro</option>
                              <option value="F">Falso</option>
                            </select>
                          </label>
                          <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete /></button>
                        </div>
                      ))}
                    </div>
                  )}

                  {q.tipo === "completar_lacunas" && (
                    <div className="flex flex-col gap-3">
                      <div className="text-xs bg-base-300 p-2 rounded">
                        No enunciado, use <kbd className="kbd kbd-xs">___</kbd> (três underscores) para cada lacuna — aparecerá como uma linha em branco para o aluno preencher.
                      </div>
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <div>
                            <span className="text-sm font-medium">Banco de palavras</span>
                            <span className="text-xs text-base-content/50 ml-2">(opcional — palavras que o aluno pode usar)</span>
                          </div>
                          <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}><MdAdd /> Palavra</button>
                        </div>
                        {q.opcoes.map((o, oIdx) => (
                          <div key={oIdx} className="flex gap-2 mb-1 items-center">
                            <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Palavra ou expressão ${oIdx + 1}`} />
                            <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete /></button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {q.tipo === "multipla_escolha" && (
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium">Alternativas</span>
                        <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}><MdAdd /> Alternativa</button>
                      </div>
                      {q.opcoes.map((o, oIdx) => (
                        <div key={oIdx} className="flex gap-2 mb-2 items-center">
                          <span className="text-sm w-6 text-base-content/60">{String.fromCharCode(97 + oIdx)})</span>
                          <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Alternativa ${oIdx + 1}`} />
                          <label className="flex items-center gap-1 text-sm cursor-pointer">
                            <input type="checkbox" className="checkbox checkbox-sm" checked={o.correta} onChange={(e) => updateOpcao(idx, oIdx, "correta", e.target.checked)} />
                            Correta
                          </label>
                          <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete /></button>
                        </div>
                      ))}
                    </div>
                  )}

                  {q.tipo === "associacao" && (
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <div>
                          <span className="text-sm font-medium">Pares de correspondência</span>
                          <p className="text-xs text-base-content/50">Coluna A: numerada (1, 2, 3...) · Coluna B: letrada (A, B, C...) — o aluno escreve o número correspondente</p>
                        </div>
                        <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}><MdAdd /> Par</button>
                      </div>
                      <div className="grid grid-cols-[auto_1fr_auto_1fr_auto] gap-x-2 gap-y-2 items-center mb-1">
                        <span />
                        <span className="text-xs font-semibold text-base-content/60">Coluna A</span>
                        <span />
                        <span className="text-xs font-semibold text-base-content/60">Coluna B</span>
                        <span />
                      </div>
                      {q.opcoes.map((o, oIdx) => (
                        <div key={oIdx} className="grid grid-cols-[auto_1fr_auto_1fr_auto] gap-x-2 gap-y-1 items-center mb-1">
                          <span className="text-sm text-base-content/60">{oIdx + 1}.</span>
                          <input className="input input-sm" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Ex: Cachorro`} />
                          <span className="text-base-content/30 text-xs">↔</span>
                          <input className="input input-sm" value={o.par ?? ""} onChange={(e) => updateOpcao(idx, oIdx, "par", e.target.value)} placeholder={`Ex: Canino`} />
                          <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete /></button>
                        </div>
                      ))}
                    </div>
                  )}

                  {q.tipo === "ordenar" && (
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <div>
                          <span className="text-sm font-medium">Itens para sequenciar</span>
                          <p className="text-xs text-base-content/50">Digite os itens <strong>na ordem que aparecerão na prova</strong>. O aluno deve numerá-los na sequência correta.</p>
                        </div>
                        <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}><MdAdd /> Item</button>
                      </div>
                      {q.opcoes.map((o, oIdx) => (
                        <div key={oIdx} className="flex gap-2 mb-2 items-center">
                          <span className="text-sm text-base-content/40 whitespace-nowrap">(   )</span>
                          <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Item ${oIdx + 1}`} />
                          <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete /></button>
                        </div>
                      ))}
                    </div>
                  )}

                  <details>
                    <summary className="cursor-pointer text-xs opacity-60 hover:opacity-100 mt-2">Tags e dificuldade</summary>
                    <div className="grid grid-cols-2 gap-3 mt-2">
                      <fieldset className="fieldset">
                        <legend className="fieldset-legend">Tags</legend>
                        <input className="input w-full input-sm" value={q.tags} onChange={(e) => updateQuestao(idx, "tags", e.target.value)} placeholder="Ex: frações, 6º ano" />
                      </fieldset>
                      <fieldset className="fieldset">
                        <legend className="fieldset-legend">Dificuldade</legend>
                        <select className="select w-full select-sm" value={q.dificuldade} onChange={(e) => updateQuestao(idx, "dificuldade", e.target.value)}>
                          <option value="fácil">Fácil</option>
                          <option value="médio">Médio</option>
                          <option value="difícil">Difícil</option>
                        </select>
                      </fieldset>
                    </div>
                  </details>
                </div>
              </div>
            ))}
          </div>
        </>
      )}

      <div className="flex justify-end gap-3 mt-6">
        <button className="btn" onClick={onClose}>Cancelar</button>
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? <span className="loading loading-spinner loading-sm" /> : "Salvar"}
        </button>
      </div>

      {bancoModal && (
        <div className="modal modal-open">
          <div className="modal-box max-w-2xl">
            <h3 className="font-bold text-lg mb-4">Importar do Banco de Questões</h3>
            <div className="overflow-x-auto max-h-96">
              <table className="table table-zebra w-full">
                <thead>
                  <tr>
                    <th>Enunciado</th>
                    <th>Tipo</th>
                    <th>Dificuldade</th>
                    <th>Tags</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {bancoQuestoes.map((q) => (
                    <tr key={q.id}>
                      <td className="max-w-xs truncate">{q.enunciado.replace(/<[^>]*>/g, "")}</td>
                      <td>{q.tipo}</td>
                      <td>{q.dificuldade}</td>
                      <td>{q.tags}</td>
                      <td>
                        <button className="btn btn-xs btn-primary" onClick={() => importarDoBanco(q.id)}>Importar</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setBancoModal(false)}>Fechar</button>
            </div>
          </div>
        </div>
      )}

      {versaoModal && (
        <div className="modal modal-open">
          <div className="modal-box max-w-sm">
            <h3 className="font-bold text-lg mb-4">Exportar Versão Embaralhada</h3>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Versão (ex: A, B, C)</legend>
              <input className="input w-full" value={versao} onChange={(e) => setVersao(e.target.value)} placeholder="A" />
            </fieldset>
            <div className="modal-action">
              <button className="btn" onClick={() => setVersaoModal(false)}>Cancelar</button>
              <button className="btn btn-primary" onClick={handleExportVersao}>Exportar PDF</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

