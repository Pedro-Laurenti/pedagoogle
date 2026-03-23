"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdCopyAll, MdLibraryBooks } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import ProvaEditor from "./ProvaEditor";
import type { Prova, Materia, Turma, BancoQuestao, ToastState } from "@/types";

type Aba = "provas" | "banco";

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
        <div className="overflow-x-auto">
          <table className="table table-zebra w-full">
            <thead>
              <tr>
                <th>Título</th>
                <th>Matéria</th>
                <th>Data</th>
                <th>Valor</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {provas.map((p) => (
                <tr key={p.id}>
                  <td>
                    {p.titulo}
                    {p.is_recuperacao && <span className="badge badge-warning badge-sm ml-2">REC</span>}
                  </td>
                  <td>{materias.find((m) => m.id === p.materia_id)?.nome ?? "-"}</td>
                  <td>{p.data}</td>
                  <td>{p.valor_total} pt</td>
                  <td className="flex gap-2">
                    <button className="btn btn-sm btn-ghost" onClick={() => setEditing(p.id)}>
                      <MdEdit />
                    </button>
                    <button className="btn btn-sm btn-ghost" onClick={() => handleDuplicate(p.id)}>
                      <MdCopyAll />
                    </button>
                    <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(p.id)}>
                      <MdDelete />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
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

