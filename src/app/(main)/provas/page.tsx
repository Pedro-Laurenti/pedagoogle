"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdCopyAll, MdLibraryBooks, MdSearch } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import ProvaEditor from "./ProvaEditor";
import type { Prova, Materia, Turma, BancoQuestao, ToastState } from "@/types";

type Aba = "provas" | "banco";
type CategoriaFiltro = "todas" | "normal" | "recuperacao";

interface BancoForm {
  tipo: string; enunciado: string; opcoes: string; valor: string; tags: string; dificuldade: string;
}

const EMPTY_BANCO: BancoForm = { tipo: "dissertativa", enunciado: "", opcoes: "[]", valor: "1", tags: "", dificuldade: "médio" };

export default function ProvasPage() {
  const [provas, setProvas] = useState<Prova[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [editing, setEditing] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [aba, setAba] = useState<Aba>("provas");
  const [banco, setBanco] = useState<BancoQuestao[]>([]);
  const [bancoBusca, setBancoBusca] = useState("");
  const [bancoModal, setBancoModal] = useState(false);
  const [bancoForm, setBancoForm] = useState<BancoForm>(EMPTY_BANCO);
  const [bancoEditing, setBancoEditing] = useState<number | null>(null);
  // Filters
  const [filtroMateria, setFiltroMateria] = useState<number | null>(null);
  const [filtroCategoria, setFiltroCategoria] = useState<CategoriaFiltro>("todas");
  const [filtroBusca, setFiltroBusca] = useState("");
  const [filtroSemestre, setFiltroSemestre] = useState("");
  const [filtroBimestre, setFiltroBimestre] = useState<"" | "1" | "2" | "3" | "4">("");

  const load = useCallback(async () => {
    const [p, m, t] = await Promise.all([invokeCmd<Prova[]>("list_provas"), invokeCmd<Materia[]>("list_materias"), invokeCmd<Turma[]>("list_turmas")]);
    setProvas(p);
    setMaterias(m);
    setTurmas(t);
  }, []);

  const loadBanco = useCallback(async () => {
    setBanco(await invokeCmd<BancoQuestao[]>("list_banco_questoes"));
  }, []);

  useEffect(() => { load(); }, [load]);
  useEffect(() => { if (aba === "banco") loadBanco(); }, [aba, loadBanco]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_prova", { id });
      notify("Prova removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  async function handleDuplicate(id: number) {
    try {
      await invokeCmd("duplicate_prova", { id });
      notify("Prova duplicada.");
      load();
    } catch {
      notify("Erro ao duplicar.", "error");
    }
  }

  function abrirNovoBanco() {
    setBancoForm(EMPTY_BANCO);
    setBancoEditing(null);
    setBancoModal(true);
  }

  function abrirEditarBanco(q: BancoQuestao) {
    setBancoForm({ tipo: q.tipo, enunciado: q.enunciado, opcoes: q.opcoes, valor: String(q.valor), tags: q.tags, dificuldade: q.dificuldade });
    setBancoEditing(q.id);
    setBancoModal(true);
  }

  async function handleSaveBanco() {
    const payload = { tipo: bancoForm.tipo, enunciado: bancoForm.enunciado, opcoes: bancoForm.opcoes, valor: parseFloat(bancoForm.valor) || 1, tags: bancoForm.tags, dificuldade: bancoForm.dificuldade };
    try {
      if (bancoEditing !== null) {
        await invokeCmd("update_banco_questao", { id: bancoEditing, ...payload });
        notify("Questão atualizada.");
      } else {
        await invokeCmd("create_banco_questao", payload);
        notify("Questão criada.");
      }
      setBancoModal(false);
      loadBanco();
    } catch (e) {
      notify(`Erro: ${e}`, "error");
    }
  }

  async function handleDeleteBanco(id: number) {
    try {
      await invokeCmd("delete_banco_questao", { id });
      notify("Questão removida.");
      loadBanco();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  function dataToBimestre(data: string | null | undefined): "1" | "2" | "3" | "4" | null {
    if (!data) return null;
    const month = parseInt(data.slice(5, 7), 10);
    if (month >= 1 && month <= 3) return "1";
    if (month >= 4 && month <= 6) return "2";
    if (month >= 7 && month <= 9) return "3";
    return "4";
  }

  const provasFiltradas = provas
    .filter((p) => {
      if (filtroMateria !== null && p.materia_id !== filtroMateria) return false;
      if (filtroCategoria === "normal" && p.is_recuperacao) return false;
      if (filtroCategoria === "recuperacao" && !p.is_recuperacao) return false;
      if (filtroBusca && !p.titulo.toLowerCase().includes(filtroBusca.toLowerCase())) return false;
      if (filtroSemestre && p.data && !p.data.startsWith(filtroSemestre)) return false;
      if (filtroBimestre && dataToBimestre(p.data) !== filtroBimestre) return false;
      return true;
    })
    .sort((a, b) => (b.data || "").localeCompare(a.data || ""));

  const bancoFiltrado = banco.filter((q) => {
    const t = bancoBusca.toLowerCase();
    return !t || q.enunciado.toLowerCase().includes(t) || q.tags.toLowerCase().includes(t);
  });

  if (creating || editing !== null) {
    return (
      <ProvaEditor
        provaId={editing}
        materias={materias}
        turmas={turmas}
        onClose={() => { setCreating(false); setEditing(null); load(); }}
        onNotify={notify}
      />
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Provas</h1>
        {aba === "provas" ? (
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            <MdAdd size={20} /> Nova Prova
          </button>
        ) : (
          <button className="btn btn-primary" onClick={abrirNovoBanco}>
            <MdAdd size={20} /> Nova Questão
          </button>
        )}
      </div>

      <div className="tabs tabs-boxed mb-6 w-fit">
        <button className={`tab ${aba === "provas" ? "tab-active" : ""}`} onClick={() => setAba("provas")}>Provas</button>
        <button className={`tab ${aba === "banco" ? "tab-active" : ""}`} onClick={() => setAba("banco")}>
          <MdLibraryBooks className="mr-1" /> Banco de Questões
        </button>
      </div>

      {aba === "provas" && (
        <div>
          {/* Filter bar */}
          <div className="flex flex-wrap gap-3 mb-6 p-4 bg-base-200 rounded-xl">
            <div className="relative flex-1 min-w-48">
              <MdSearch className="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/50" size={18} />
              <input
                className="input pl-9 w-full"
                placeholder="Buscar por título..."
                value={filtroBusca}
                onChange={(e) => setFiltroBusca(e.target.value)}
              />
            </div>
            <select
              className="select"
              value={filtroMateria ?? ""}
              onChange={(e) => setFiltroMateria(e.target.value === "" ? null : Number(e.target.value))}
            >
              <option value="">Todas as matérias</option>
              {materias.map((m) => <option key={m.id} value={m.id}>{m.nome}</option>)}
            </select>
            <select
              className="select"
              value={filtroSemestre}
              onChange={(e) => setFiltroSemestre(e.target.value)}
            >
              <option value="">Todos os semestres</option>
              {[...new Set(provas.map((p) => p.data?.slice(0, 7)).filter(Boolean))].sort().reverse().map((s) => (
                <option key={s} value={s!}>{s}</option>
              ))}
            </select>
            <select
              className="select"
              value={filtroBimestre}
              onChange={(e) => setFiltroBimestre(e.target.value as "" | "1" | "2" | "3" | "4")}
            >
              <option value="">Todos os bimestres</option>
              <option value="1">1º Bimestre (Jan–Mar)</option>
              <option value="2">2º Bimestre (Abr–Jun)</option>
              <option value="3">3º Bimestre (Jul–Set)</option>
              <option value="4">4º Bimestre (Out–Dez)</option>
            </select>
            <select
              className="select"
              value={filtroCategoria}
              onChange={(e) => setFiltroCategoria(e.target.value as CategoriaFiltro)}
            >
              <option value="todas">Todas</option>
              <option value="normal">Provas normais</option>
              <option value="recuperacao">Recuperação</option>
            </select>
          </div>

          {/* Cards */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {provasFiltradas.map((p) => {
              const materia = materias.find((m) => m.id === p.materia_id);
              return (
                <div key={p.id} className="card bg-base-200 shadow hover:shadow-md transition-shadow">
                  <div className="card-body gap-2">
                    <div className="flex items-start justify-between gap-2">
                      <h3 className="card-title text-base leading-tight">{p.titulo}</h3>
                      {p.is_recuperacao && (
                        <span className="badge badge-warning badge-sm shrink-0">Recuperação</span>
                      )}
                    </div>
                    {materia && (
                      <div className="flex items-center gap-2 text-sm">
                        <span
                          className="inline-block w-3 h-3 rounded-full shrink-0"
                          style={{ backgroundColor: materia.cor }}
                        />
                        <span className="text-base-content/70">{materia.nome}</span>
                      </div>
                    )}
                    <div className="flex items-center gap-4 text-xs text-base-content/60 mt-1">
                      {p.data && <span>📅 {p.data}</span>}
                      <span>📝 {p.questoes_count} questão(ões)</span>
                      <span>💯 {p.valor_total} pt</span>
                    </div>
                    <div className="card-actions justify-end mt-2">
                      <button className="btn btn-xs btn-ghost" onClick={() => setEditing(p.id)}><MdEdit size={14} /> Editar</button>
                      <button className="btn btn-xs btn-ghost" onClick={() => handleDuplicate(p.id)}><MdCopyAll size={14} /></button>
                      <button className="btn btn-xs btn-ghost text-error" onClick={() => handleDelete(p.id)}><MdDelete size={14} /></button>
                    </div>
                  </div>
                </div>
              );
            })}
            {provasFiltradas.length === 0 && (
              <div className="col-span-full text-center py-12 text-base-content/40">Nenhuma prova encontrada.</div>
            )}
          </div>
        </div>
      )}

      {aba === "banco" && (
        <div>
          <fieldset className="fieldset mb-4">
            <legend className="fieldset-legend">Filtrar por enunciado ou tags</legend>
            <input className="input w-full" value={bancoBusca} onChange={(e) => setBancoBusca(e.target.value)} placeholder="Digite para filtrar..." />
          </fieldset>
          <div className="overflow-x-auto">
            <table className="table table-zebra w-full">
              <thead>
                <tr>
                  <th>Enunciado</th>
                  <th>Tipo</th>
                  <th>Dificuldade</th>
                  <th>Tags</th>
                  <th>Valor</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {bancoFiltrado.map((q) => (
                  <tr key={q.id}>
                    <td className="max-w-xs truncate">{q.enunciado.replace(/<[^>]*>/g, "")}</td>
                    <td>{q.tipo}</td>
                    <td>{q.dificuldade}</td>
                    <td>{q.tags}</td>
                    <td>{q.valor} pt</td>
                    <td className="flex gap-2">
                      <button className="btn btn-sm btn-ghost" onClick={() => abrirEditarBanco(q)}><MdEdit /></button>
                      <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDeleteBanco(q.id)}><MdDelete /></button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {bancoModal && (
        <div className="modal modal-open">
          <div className="modal-box max-w-lg">
            <h3 className="font-bold text-lg mb-4">{bancoEditing !== null ? "Editar Questão" : "Nova Questão"}</h3>
            <div className="flex flex-col gap-3">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Tipo</legend>
                <select className="select w-full" value={bancoForm.tipo} onChange={(e) => setBancoForm({ ...bancoForm, tipo: e.target.value })}>
                  <option value="dissertativa">Dissertativa</option>
                  <option value="multipla_escolha">Múltipla Escolha</option>
                  <option value="verdadeiro_falso">Verdadeiro ou Falso</option>
                  <option value="completar_lacunas">Completar Lacunas</option>
                  <option value="associacao">Associação</option>
                  <option value="ordenar">Ordenar/Sequenciar</option>
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Enunciado</legend>
                <textarea className="textarea w-full" rows={3} value={bancoForm.enunciado} onChange={(e) => setBancoForm({ ...bancoForm, enunciado: e.target.value })} />
              </fieldset>
              <div className="grid grid-cols-2 gap-3">
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Valor (pts)</legend>
                  <input type="number" step="0.5" min="0" className="input w-full" value={bancoForm.valor} onChange={(e) => setBancoForm({ ...bancoForm, valor: e.target.value })} />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Dificuldade</legend>
                  <select className="select w-full" value={bancoForm.dificuldade} onChange={(e) => setBancoForm({ ...bancoForm, dificuldade: e.target.value })}>
                    <option value="fácil">Fácil</option>
                    <option value="médio">Médio</option>
                    <option value="difícil">Difícil</option>
                  </select>
                </fieldset>
              </div>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Tags</legend>
                <input className="input w-full" value={bancoForm.tags} onChange={(e) => setBancoForm({ ...bancoForm, tags: e.target.value })} placeholder="Ex: matemática, frações, 6º ano" />
              </fieldset>
            </div>
            <div className="modal-action">
              <button className="btn" onClick={() => setBancoModal(false)}>Cancelar</button>
              <button className="btn btn-primary" onClick={handleSaveBanco}>Salvar</button>
            </div>
          </div>
        </div>
      )}

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}

