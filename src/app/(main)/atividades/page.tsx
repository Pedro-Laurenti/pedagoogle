"use client";
import { useState, useEffect, useCallback } from "react";
import {
  MdAdd, MdEdit, MdDelete, MdCopyAll, MdSearch,
  MdFilterList, MdSchool, MdCalendarToday,
} from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import AtividadeEditor from "./AtividadeEditor";
import type { Atividade, Materia, Turma, Configuracoes, ToastState } from "@/types";

export default function AtividadesPage() {
  const [atividades, setAtividades] = useState<Atividade[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [editing, setEditing] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [deleteId, setDeleteId] = useState<number | null>(null);

  const [filtroMateria, setFiltroMateria] = useState<string>("");
  const [filtroBimestre, setFiltroBimestre] = useState<string>("");
  const [filtroAno, setFiltroAno] = useState<string>("");
  const [filtroBusca, setFiltroBusca] = useState<string>("");

  const load = useCallback(async () => {
    const [a, m, t, cfg] = await Promise.all([
      invokeCmd<Atividade[]>("list_atividades"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setAtividades(a);
    setMaterias(m);
    setTurmas(t);
    setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_atividade", { id });
      notify("Atividade removida.");
      setDeleteId(null);
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  async function handleDuplicate(id: number) {
    try {
      await invokeCmd("duplicate_atividade", { id });
      notify("Atividade duplicada.");
      load();
    } catch {
      notify("Erro ao duplicar.", "error");
    }
  }

  const anosLetivos = [...new Set(atividades.map((a) => a.ano_letivo).filter(Boolean))].sort().reverse();

  const atividadesFiltradas = atividades.filter((a) => {
    if (filtroMateria && String(a.materia_id) !== filtroMateria) return false;
    if (filtroBimestre && String(a.bimestre) !== filtroBimestre) return false;
    if (filtroAno && a.ano_letivo !== filtroAno) return false;
    if (filtroBusca && !a.titulo.toLowerCase().includes(filtroBusca.toLowerCase())) return false;
    return true;
  });

  if (creating || editing !== null) {
    return (
      <AtividadeEditor
        atividadeId={editing}
        materias={materias}
        turmas={turmas}
        usarTurmas={config?.usar_turmas ?? false}
        onClose={() => { setCreating(false); setEditing(null); load(); }}
        onNotify={notify}
      />
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Atividades</h1>
        <button className="btn btn-primary" onClick={() => setCreating(true)}>
          <MdAdd size={20} /> Nova Atividade
        </button>
      </div>

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
        <div className="flex items-center gap-1.5">
          <MdSchool size={16} className="text-base-content/50 shrink-0" />
          <select
            className="select"
            value={filtroMateria}
            onChange={(e) => setFiltroMateria(e.target.value)}
          >
            <option value="">Todas as matérias</option>
            {materias.map((m) => <option key={m.id} value={m.id}>{m.nome}</option>)}
          </select>
        </div>
        <div className="flex items-center gap-1.5">
          <MdFilterList size={16} className="text-base-content/50 shrink-0" />
          <select
            className="select"
            value={filtroBimestre}
            onChange={(e) => setFiltroBimestre(e.target.value)}
          >
            <option value="">Todos os bimestres</option>
            <option value="1">1º Bimestre</option>
            <option value="2">2º Bimestre</option>
            <option value="3">3º Bimestre</option>
            <option value="4">4º Bimestre</option>
          </select>
        </div>
        <div className="flex items-center gap-1.5">
          <MdCalendarToday size={16} className="text-base-content/50 shrink-0" />
          <select
            className="select"
            value={filtroAno}
            onChange={(e) => setFiltroAno(e.target.value)}
          >
            <option value="">Todos os anos</option>
            {anosLetivos.map((a) => <option key={a} value={a}>{a}</option>)}
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {atividadesFiltradas.map((a) => {
          const materia = materias.find((m) => m.id === a.materia_id);
          return (
            <div key={a.id} className="card bg-base-200 shadow hover:shadow-md transition-shadow">
              <div className="card-body gap-2">
                <div className="flex items-start justify-between gap-2">
                  <h3 className="card-title text-base leading-tight">{a.titulo}</h3>
                  <div className="flex flex-col items-end gap-1 shrink-0">
                    <span className="badge badge-outline badge-sm">{a.bimestre}º Bim</span>
                    {a.vale_nota && <span className="badge badge-success badge-sm">Vale nota</span>}
                  </div>
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
                  {a.ano_letivo && <span>{a.ano_letivo}</span>}
                  <span>{a.questoes_count} questão(ões)</span>
                  {a.vale_nota && <span>{a.valor_total} pt</span>}
                </div>
                <div className="card-actions justify-end mt-2">
                  <button className="btn btn-xs btn-ghost" onClick={() => setEditing(a.id)}>
                    <MdEdit size={14} /> Editar
                  </button>
                  <button className="btn btn-xs btn-ghost" onClick={() => handleDuplicate(a.id)} title="Duplicar">
                    <MdCopyAll size={14} />
                  </button>
                  <button className="btn btn-xs btn-ghost text-error" onClick={() => setDeleteId(a.id)} title="Excluir">
                    <MdDelete size={14} />
                  </button>
                </div>
              </div>
            </div>
          );
        })}
        {atividadesFiltradas.length === 0 && (
          <div className="col-span-full text-center py-12 text-base-content/40">
            Nenhuma atividade encontrada.
          </div>
        )}
      </div>

      <Modal
        open={deleteId !== null}
        onClose={() => setDeleteId(null)}
        variant="confirm"
        color="error"
        title="Excluir atividade"
        confirmLabel="Excluir"
        onConfirm={() => deleteId !== null && handleDelete(deleteId)}
      >
        Tem certeza que deseja excluir esta atividade? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
