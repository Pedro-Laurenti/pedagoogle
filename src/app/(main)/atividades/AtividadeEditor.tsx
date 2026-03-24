"use client";
import { useState, useEffect, useCallback } from "react";
import {
  MdArrowBack, MdAdd, MdDelete, MdPictureAsPdf,
  MdArrowUpward, MdArrowDownward, MdTextFields,
} from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import RichEditor from "@/components/RichEditor";
import "katex/dist/katex.min.css";
import type { Materia, Atividade, AtividadeQuestao, QuestaoInput, OpcaoQuestao, TipoQuestao, Turma, ToastState } from "@/types";

const TIPOS: (TipoQuestao | "texto")[] = [
  "dissertativa", "multipla_escolha", "verdadeiro_falso",
  "completar_lacunas", "associacao", "ordenar",
];
const TIPO_LABELS: Record<TipoQuestao | "texto", string> = {
  dissertativa: "Dissertativa",
  multipla_escolha: "Múltipla Escolha",
  verdadeiro_falso: "Verdadeiro ou Falso",
  completar_lacunas: "Completar Lacunas",
  associacao: "Associação",
  ordenar: "Ordenar/Sequenciar",
  texto: "Bloco de Texto",
};

interface AtividadeForm {
  titulo: string;
  descricao: string;
  materia_id: string;
  bimestre: string;
  ano_letivo: string;
  valor_total: string;
  turma_id: string;
  vale_nota: boolean;
}

interface Errors {
  titulo?: string;
  materia_id?: string;
  bimestre?: string;
  ano_letivo?: string;
  valor_total?: string;
}

interface Props {
  atividadeId: number | null;
  materias: Materia[];
  turmas: Turma[];
  usarTurmas: boolean;
  onClose: () => void;
  onNotify: (message: string, type?: ToastState["type"]) => void;
}

const EMPTY_FORM: AtividadeForm = {
  titulo: "", descricao: "", materia_id: "", bimestre: "1",
  ano_letivo: new Date().getFullYear().toString(),
  valor_total: "10", turma_id: "", vale_nota: false,
};

const newQuestao = (): QuestaoInput => ({
  enunciado: "", tipo: "dissertativa", opcoes: [], valor: 0,
  linhas_resposta: 3, tempId: Date.now() + Math.random(),
});

const newTexto = (): QuestaoInput => ({
  enunciado: "", tipo: "texto", opcoes: [], valor: 0,
  linhas_resposta: 0, tempId: Date.now() + Math.random(),
});

function somaQuestoes(questoes: QuestaoInput[]) {
  return questoes.reduce((acc, q) => acc + (q.tipo === "texto" ? 0 : Number(q.valor) || 0), 0);
}

export default function AtividadeEditor({ atividadeId, materias, turmas, usarTurmas, onClose, onNotify }: Props) {
  const [form, setForm] = useState<AtividadeForm>(EMPTY_FORM);
  const [questoes, setQuestoes] = useState<QuestaoInput[]>([]);
  const [aba, setAba] = useState<"config" | "questoes">("config");
  const [errors, setErrors] = useState<Errors>({});
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    if (!atividadeId) return;
    const a = await invokeCmd<Atividade>("get_atividade", { id: atividadeId });
    setForm({
      titulo: a.titulo,
      descricao: a.descricao,
      materia_id: a.materia_id ? String(a.materia_id) : "",
      bimestre: String(a.bimestre ?? 1),
      ano_letivo: a.ano_letivo ?? "",
      valor_total: String(a.valor_total),
      turma_id: a.turma_id ? String(a.turma_id) : "",
      vale_nota: a.vale_nota,
    });
    const qs = await invokeCmd<AtividadeQuestao[]>("list_questoes_atividade", { atividadeId });
    setQuestoes(qs.map((q) => ({
      id: q.id, enunciado: q.enunciado, tipo: q.tipo,
      opcoes: q.opcoes, valor: q.valor, linhas_resposta: q.linhas_resposta,
    })));
  }, [atividadeId]);

  useEffect(() => { load(); }, [load]);

  function set(field: keyof AtividadeForm, value: string | boolean) {
    setForm((f) => ({ ...f, [field]: value }));
    setErrors((e) => ({ ...e, [field]: undefined }));
  }

  function validate(): boolean {
    const errs: Errors = {};
    if (!form.titulo.trim()) errs.titulo = "Informe o título";
    if (!form.materia_id) errs.materia_id = "Selecione uma matéria";
    if (!form.bimestre) errs.bimestre = "Selecione o bimestre";
    if (!form.ano_letivo.trim()) errs.ano_letivo = "Informe o ano letivo";
    if (form.vale_nota && (!form.valor_total || isNaN(parseFloat(form.valor_total)))) errs.valor_total = "Informe o valor total";
    setErrors(errs);
    return Object.keys(errs).length === 0;
  }

  function addQuestao() { setQuestoes((qs) => [...qs, newQuestao()]); }
  function addTexto()   { setQuestoes((qs) => [...qs, newTexto()]); }

  function removeQuestao(idx: number) { setQuestoes((qs) => qs.filter((_, i) => i !== idx)); }

  function moveUp(idx: number) {
    if (idx === 0) return;
    setQuestoes((qs) => {
      const next = [...qs];
      [next[idx - 1], next[idx]] = [next[idx], next[idx - 1]];
      return next;
    });
  }

  function moveDown(idx: number) {
    setQuestoes((qs) => {
      if (idx >= qs.length - 1) return qs;
      const next = [...qs];
      [next[idx], next[idx + 1]] = [next[idx + 1], next[idx]];
      return next;
    });
  }

  function updateQuestao(idx: number, field: keyof QuestaoInput, value: unknown) {
    setQuestoes((qs) => qs.map((q, i) => i === idx ? { ...q, [field]: value } : q));
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

  function distribuirPontos() {
    const qs = questoes.filter((q) => q.tipo !== "texto");
    if (qs.length === 0) return;
    const vt = parseFloat(form.valor_total) || 10;
    const por = Math.round((vt / qs.length) * 100) / 100;
    setQuestoes((all) => all.map((q) => q.tipo === "texto" ? q : { ...q, valor: por }));
  }

  async function handleSave() {
    if (!validate()) {
      onNotify("Corrija os campos obrigatórios.", "error");
      return;
    }
    for (let i = 0; i < questoes.length; i++) {
      const q = questoes[i];
      if (q.tipo === "multipla_escolha" && !q.opcoes.some((o) => o.correta)) {
        onNotify(`Questão ${i + 1}: marque ao menos uma opção correta`, "error");
        setAba("questoes");
        return;
      }
      if (q.tipo === "verdadeiro_falso" && q.opcoes.some((o) => !o.texto.trim())) {
        onNotify(`Questão ${i + 1}: preencha o texto de todas as afirmações`, "error");
        setAba("questoes");
        return;
      }
    }
    setSaving(true);
    try {
      const payload = {
        titulo: form.titulo,
        descricao: form.descricao,
        materiaId: form.materia_id ? Number(form.materia_id) : null,
        bimestre: Number(form.bimestre),
        anoLetivo: form.ano_letivo,
        turmaId: form.turma_id ? Number(form.turma_id) : null,
        valorTotal: form.vale_nota ? (parseFloat(form.valor_total) || 10) : 0,
        valeNota: form.vale_nota,
      };
      let id = atividadeId;
      if (!id) {
        id = await invokeCmd<number>("create_atividade", payload);
      } else {
        await invokeCmd("update_atividade", { id, ...payload });
      }
      await invokeCmd("replace_questoes_atividade", { atividadeId: id, questoes });
      onNotify("Atividade salva.");
      onClose();
    } catch (e) {
      onNotify(`Erro ao salvar: ${e}`, "error");
    } finally {
      setSaving(false);
    }
  }

  async function handleExportPdf() {
    if (!atividadeId) { onNotify("Salve a atividade antes de exportar.", "warning"); return; }
    const safeName = form.titulo.replace(/[^a-zA-Z0-9\s]/g, "").trim().replace(/\s+/g, "_") || "atividade";
    const filePath = await save({
      defaultPath: `${safeName}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!filePath) return;
    try {
      await invokeCmd("export_atividade_pdf", { id: atividadeId, path: filePath });
      onNotify("PDF exportado com sucesso.");
    } catch (e) {
      onNotify(`Erro ao exportar: ${e}`, "error");
    }
  }

  const soma = somaQuestoes(questoes);
  const valorTotal = parseFloat(form.valor_total) || 10;
  const somaOk = !form.vale_nota || Math.abs(soma - valorTotal) < 0.01;
  const numQuestoes = questoes.filter((q) => q.tipo !== "texto").length;

  return (
    <div>
      <div className="flex items-center gap-3 mb-6 flex-wrap">
        <button className="btn btn-ghost" onClick={onClose}><MdArrowBack size={20} /></button>
        <div className="flex-1">
          <h1 className="text-2xl font-bold">{atividadeId ? "Editar Atividade" : "Nova Atividade"}</h1>
          {form.titulo && <p className="text-sm text-base-content/60 mt-0.5">{form.titulo}</p>}
        </div>
        {atividadeId && (
          <button className="btn btn-sm btn-outline gap-1" onClick={handleExportPdf}>
            <MdPictureAsPdf size={18} /> PDF
          </button>
        )}
      </div>

      <div className="tabs tabs-boxed mb-6 w-fit">
        <button className={`tab ${aba === "config" ? "tab-active" : ""}`} onClick={() => setAba("config")}>
          Configurações
        </button>
        <button className={`tab ${aba === "questoes" ? "tab-active" : ""}`} onClick={() => setAba("questoes")}>
          Questões
          {numQuestoes > 0 && <span className="badge badge-sm ml-2">{numQuestoes}</span>}
        </button>
      </div>

      {aba === "config" && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <fieldset className="fieldset md:col-span-2">
            <legend className="fieldset-legend">
              Título <span className="text-error">*</span>
            </legend>
            <input
              className={`input w-full${errors.titulo ? " input-error" : ""}`}
              value={form.titulo}
              onChange={(e) => set("titulo", e.target.value)}
              placeholder="Ex: Lista de Exercícios 1"
            />
            {errors.titulo && <p className="text-error text-sm mt-1">{errors.titulo}</p>}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              Matéria <span className="text-error">*</span>
            </legend>
            <select
              className={`select w-full${errors.materia_id ? " select-error" : ""}`}
              value={form.materia_id}
              onChange={(e) => set("materia_id", e.target.value)}
            >
              <option value="">Selecione</option>
              {materias.map((m) => <option key={m.id} value={m.id}>{m.nome}</option>)}
            </select>
            {errors.materia_id && <p className="text-error text-sm mt-1">{errors.materia_id}</p>}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              Bimestre <span className="text-error">*</span>
            </legend>
            <select
              className={`select w-full${errors.bimestre ? " select-error" : ""}`}
              value={form.bimestre}
              onChange={(e) => set("bimestre", e.target.value)}
            >
              <option value="1">1º Bimestre</option>
              <option value="2">2º Bimestre</option>
              <option value="3">3º Bimestre</option>
              <option value="4">4º Bimestre</option>
            </select>
            {errors.bimestre && <p className="text-error text-sm mt-1">{errors.bimestre}</p>}
          </fieldset>

          <fieldset className="fieldset">
            <legend className="fieldset-legend">
              Ano Letivo <span className="text-error">*</span>
            </legend>
            <input
              className={`input w-full${errors.ano_letivo ? " input-error" : ""}`}
              value={form.ano_letivo}
              onChange={(e) => set("ano_letivo", e.target.value)}
              placeholder="Ex: 2025"
            />
            {errors.ano_letivo && <p className="text-error text-sm mt-1">{errors.ano_letivo}</p>}
          </fieldset>

          {usarTurmas && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Turma</legend>
              <select
                className="select w-full"
                value={form.turma_id}
                onChange={(e) => set("turma_id", e.target.value)}
              >
                <option value="">Todas as turmas</option>
                {turmas.map((t) => <option key={t.id} value={t.id}>{t.nome}</option>)}
              </select>
            </fieldset>
          )}

          <fieldset className="fieldset">
            <legend className="fieldset-legend">Avaliação</legend>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                className="checkbox"
                checked={form.vale_nota}
                onChange={(e) => set("vale_nota", e.target.checked)}
              />
              <span>Vale nota</span>
            </label>
          </fieldset>

          {form.vale_nota && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">
                Valor Total <span className="text-error">*</span>
              </legend>
              <input
                type="number"
                step="0.5"
                min="0"
                className={`input w-full${errors.valor_total ? " input-error" : ""}`}
                value={form.valor_total}
                onChange={(e) => set("valor_total", e.target.value)}
              />
              {errors.valor_total && <p className="text-error text-sm mt-1">{errors.valor_total}</p>}
            </fieldset>
          )}

          <fieldset className="fieldset md:col-span-2">
            <legend className="fieldset-legend">Instruções / Descrição</legend>
            <RichEditor
              value={form.descricao}
              onChange={(html) => set("descricao", html)}
              placeholder="Instruções gerais para os alunos"
              minHeight={100}
            />
          </fieldset>
        </div>
      )}

      {aba === "questoes" && (
        <>
          <div className="flex items-center justify-between mb-4 flex-wrap gap-2">
            <div className="flex items-center gap-3">
              {form.vale_nota && (
                <>
                  <span className={`badge ${somaOk ? "badge-success" : "badge-warning"}`}>
                    {soma.toFixed(2)} / {valorTotal} pt
                  </span>
                  {questoes.length > 0 && (
                    <button className="btn btn-xs btn-ghost" onClick={distribuirPontos}>
                      Distribuir igualmente
                    </button>
                  )}
                </>
              )}
            </div>
            <div className="flex gap-2">
              <button className="btn btn-sm btn-outline gap-1" onClick={addTexto}>
                <MdTextFields size={16} /> Texto
              </button>
              <button className="btn btn-sm btn-primary gap-1" onClick={addQuestao}>
                <MdAdd size={16} /> Questão
              </button>
            </div>
          </div>

          {form.vale_nota && !somaOk && questoes.filter((q) => q.tipo !== "texto").length > 0 && (
            <div className="alert alert-warning mb-4 text-sm">
              A soma das questões ({soma.toFixed(2)}) não corresponde ao valor total ({valorTotal}).
            </div>
          )}

          <div className="flex flex-col gap-4">
            {questoes.map((q, idx) => {
              const isTexto = q.tipo === "texto";
              const numLabel = isTexto ? null : questoes.slice(0, idx).filter((x) => x.tipo !== "texto").length + 1;
              return (
                <div key={q.id ?? q.tempId ?? idx} className={`card shadow ${isTexto ? "bg-base-100 border border-base-300" : "bg-base-200"}`}>
                  <div className="card-body gap-3">
                    <div className="flex items-center gap-2">
                      <div className="flex flex-col gap-0.5">
                        <button className="btn btn-xs btn-ghost p-0.5" onClick={() => moveUp(idx)} disabled={idx === 0}>
                          <MdArrowUpward size={14} />
                        </button>
                        <button className="btn btn-xs btn-ghost p-0.5" onClick={() => moveDown(idx)} disabled={idx === questoes.length - 1}>
                          <MdArrowDownward size={14} />
                        </button>
                      </div>
                      <span className="font-medium flex-1">
                        {isTexto ? (
                          <span className="text-base-content/60 text-sm italic">Bloco de texto</span>
                        ) : (
                          `Questão ${numLabel}`
                        )}
                      </span>
                      {!isTexto && (
                        <fieldset className="fieldset p-0 border-0">
                          <select
                            className="select select-sm"
                            value={q.tipo}
                            onChange={(e) => updateQuestao(idx, "tipo", e.target.value as TipoQuestao)}
                          >
                            {TIPOS.map((t) => <option key={t} value={t}>{TIPO_LABELS[t]}</option>)}
                          </select>
                        </fieldset>
                      )}
                      {!isTexto && form.vale_nota && (
                        <div className="flex items-center gap-1">
                          <input
                            type="number"
                            step="0.5"
                            min="0"
                            className="input input-sm w-20"
                            value={q.valor}
                            onChange={(e) => updateQuestao(idx, "valor", parseFloat(e.target.value) || 0)}
                            title="Pontos"
                          />
                          <span className="text-xs text-base-content/60">pt</span>
                        </div>
                      )}
                      <button className="btn btn-xs btn-ghost text-error" onClick={() => removeQuestao(idx)}>
                        <MdDelete size={14} />
                      </button>
                    </div>

                    <fieldset className="fieldset">
                      <legend className="fieldset-legend">
                        {isTexto ? "Conteúdo do bloco" : "Enunciado"}
                      </legend>
                      <RichEditor
                        value={q.enunciado}
                        onChange={(html) => updateQuestao(idx, "enunciado", html)}
                        placeholder={isTexto ? "Digite o texto ou instrução..." : (q.tipo === "completar_lacunas" ? 'Ex: "O Brasil foi descoberto em ___ por ___"' : "Digite o enunciado da questão")}
                        minHeight={80}
                      />
                    </fieldset>

                    {q.tipo === "dissertativa" && (
                      <fieldset className="fieldset">
                        <legend className="fieldset-legend">Linhas para resposta</legend>
                        <input
                          type="number"
                          min="1"
                          max="20"
                          className="input input-sm w-24"
                          value={q.linhas_resposta}
                          onChange={(e) => updateQuestao(idx, "linhas_resposta", parseInt(e.target.value) || 3)}
                        />
                      </fieldset>
                    )}

                    {q.tipo === "verdadeiro_falso" && (
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium">Afirmações</span>
                          <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}>
                            <MdAdd size={12} /> Afirmação
                          </button>
                        </div>
                        {q.opcoes.map((o, oIdx) => (
                          <div key={oIdx} className="flex gap-2 mb-2 items-center">
                            <span className="text-sm w-6 text-base-content/60">{String.fromCharCode(97 + oIdx)})</span>
                            <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Afirmação ${oIdx + 1}`} />
                            <select className="select select-xs" value={o.correta ? "V" : "F"} onChange={(e) => updateOpcao(idx, oIdx, "correta", e.target.value === "V")}>
                              <option value="V">Verdadeiro</option>
                              <option value="F">Falso</option>
                            </select>
                            <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete size={12} /></button>
                          </div>
                        ))}
                      </div>
                    )}

                    {q.tipo === "multipla_escolha" && (
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium">Alternativas</span>
                          <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}>
                            <MdAdd size={12} /> Alternativa
                          </button>
                        </div>
                        {q.opcoes.map((o, oIdx) => (
                          <div key={oIdx} className="flex gap-2 mb-2 items-center">
                            <span className="text-sm w-6 text-base-content/60">{String.fromCharCode(97 + oIdx)})</span>
                            <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Alternativa ${oIdx + 1}`} />
                            <label className="flex items-center gap-1 text-sm cursor-pointer">
                              <input type="checkbox" className="checkbox checkbox-sm" checked={o.correta} onChange={(e) => updateOpcao(idx, oIdx, "correta", e.target.checked)} />
                              Correta
                            </label>
                            <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete size={12} /></button>
                          </div>
                        ))}
                      </div>
                    )}

                    {q.tipo === "completar_lacunas" && (
                      <div className="flex flex-col gap-2">
                        <div className="text-xs bg-base-300 p-2 rounded">
                          Use <kbd className="kbd kbd-xs">___</kbd> no enunciado para cada lacuna.
                        </div>
                        <div>
                          <div className="flex items-center justify-between mb-1">
                            <span className="text-sm font-medium">Banco de palavras <span className="opacity-50">(opcional)</span></span>
                            <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}>
                              <MdAdd size={12} /> Palavra
                            </button>
                          </div>
                          {q.opcoes.map((o, oIdx) => (
                            <div key={oIdx} className="flex gap-2 mb-1 items-center">
                              <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Palavra ${oIdx + 1}`} />
                              <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete size={12} /></button>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {q.tipo === "associacao" && (
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium">Pares</span>
                          <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}>
                            <MdAdd size={12} /> Par
                          </button>
                        </div>
                        {q.opcoes.map((o, oIdx) => (
                          <div key={oIdx} className="grid grid-cols-[auto_1fr_auto_1fr_auto] gap-x-2 gap-y-1 items-center mb-1">
                            <span className="text-sm text-base-content/60">{oIdx + 1}.</span>
                            <input className="input input-sm" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder="Coluna A" />
                            <span className="text-base-content/30 text-xs">↔</span>
                            <input className="input input-sm" value={o.par ?? ""} onChange={(e) => updateOpcao(idx, oIdx, "par", e.target.value)} placeholder="Coluna B" />
                            <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete size={12} /></button>
                          </div>
                        ))}
                      </div>
                    )}

                    {q.tipo === "ordenar" && (
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium">Itens para sequenciar</span>
                          <button className="btn btn-xs btn-outline" onClick={() => addOpcao(idx)}>
                            <MdAdd size={12} /> Item
                          </button>
                        </div>
                        {q.opcoes.map((o, oIdx) => (
                          <div key={oIdx} className="flex gap-2 mb-2 items-center">
                            <span className="text-sm text-base-content/40">(   )</span>
                            <input className="input input-sm flex-1" value={o.texto} onChange={(e) => updateOpcao(idx, oIdx, "texto", e.target.value)} placeholder={`Item ${oIdx + 1}`} />
                            <button className="btn btn-xs btn-ghost text-error" onClick={() => removeOpcao(idx, oIdx)}><MdDelete size={12} /></button>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}

            {questoes.length === 0 && (
              <div className="text-center py-10 text-base-content/40">
                Use os botões acima para adicionar questões ou blocos de texto.
              </div>
            )}
          </div>
        </>
      )}

      <div className="flex justify-end gap-3 mt-8">
        <button className="btn" onClick={onClose}>Cancelar</button>
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? <span className="loading loading-spinner loading-sm" /> : "Salvar"}
        </button>
      </div>
    </div>
  );
}
