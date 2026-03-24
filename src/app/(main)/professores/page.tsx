"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import InputMultiSelect from "@/components/inputs/InputMultiSelect";
import type { Professor, Materia, Turma, Configuracoes, ToastState } from "@/types";

interface ProfessorForm {
  nome: string;
  aulas_por_semana: number;
  [k: string]: unknown;
}

const EMPTY: ProfessorForm = { nome: "", aulas_por_semana: 0 };

export default function ProfessoresPage() {
  const [professores, setProfessores] = useState<Professor[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<ProfessorForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [selMaterias, setSelMaterias] = useState<(string | number)[]>([]);
  const [selTurmas, setSelTurmas] = useState<(string | number)[]>([]);

  const load = useCallback(async () => {
    const [p, m, t, cfg] = await Promise.all([
      invokeCmd<Professor[]>("list_professores"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setProfessores(p);
    setMaterias(m);
    setTurmas(t);
    setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setSelMaterias([]);
    setSelTurmas([]);
    setModal(true);
  }

  async function openEdit(p: Professor) {
    setEditing(p.id);
    setForm({ nome: p.nome, aulas_por_semana: p.aulas_por_semana });
    const [pm, pt] = await Promise.all([
      invokeCmd<number[]>("list_professor_materias", { professorId: p.id }),
      invokeCmd<number[]>("list_professor_turmas", { professorId: p.id }),
    ]);
    setSelMaterias(pm);
    setSelTurmas(pt);
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      let id = editing;
      if (editing !== null) {
        await invokeCmd("update_professor", { id: editing, nome: form.nome, aulasPorSemana: form.aulas_por_semana });
        notify("Professor atualizado.");
      } else {
        id = await invokeCmd<number>("create_professor", { nome: form.nome, aulasPorSemana: form.aulas_por_semana });
        notify("Professor criado.");
      }
      if (id !== null) {
        await invokeCmd("set_professor_materias", { professorId: id, materiaIds: selMaterias });
        if (config?.usar_turmas) {
          await invokeCmd("set_professor_turmas", { professorId: id, turmaIds: selTurmas });
        }
      }
      setModal(false);
      load();
    } catch (err) {
      notify(String(err) || "Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_professor", { id });
      notify("Professor removido.");
      setDeleteId(null);
      load();
    } catch (err) {
      notify(String(err), "error");
    }
  }

  if (!config?.usar_professores) {
    return <div className="text-center text-base-content/50 mt-16">Módulo de professores desabilitado nas configurações.</div>;
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Professores</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Novo Professor
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Nome</th>
              <th>Aulas/sem.</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {professores.map((p) => (
              <tr key={p.id}>
                <td>{p.nome}</td>
                <td>{p.aulas_por_semana}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(p)}><MdEdit /></button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => setDeleteId(p.id)}><MdDelete /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Modal open={modal} onClose={() => setModal(false)} title={`${editing !== null ? "Editar" : "Novo"} Professor`} size="lg">
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Nome *</legend>
              <input className="input w-full" value={form.nome} onChange={(e) => setForm({ ...form, nome: e.target.value })} required />
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Aulas por semana</legend>
              <input type="number" className="input w-full" min={0} value={form.aulas_por_semana} onChange={(e) => setForm({ ...form, aulas_por_semana: parseInt(e.target.value) || 0 })} />
            </fieldset>
          </div>
          <InputMultiSelect
            label="Matérias"
            options={materias.map(m => ({ value: m.id, label: m.nome }))}
            value={selMaterias}
            onChange={setSelMaterias}
          />
          {config?.usar_turmas && (
            <InputMultiSelect
              label="Turmas"
              options={turmas.map(t => ({ value: t.id, label: t.nome }))}
              value={selTurmas}
              onChange={setSelTurmas}
            />
          )}
          <div className="modal-action">
            <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
            <button type="submit" className="btn btn-primary">Salvar</button>
          </div>
        </form>
      </Modal>

      <Modal
        open={deleteId !== null}
        onClose={() => setDeleteId(null)}
        title="Confirmar exclusão"
        variant="confirm"
        color="error"
        confirmLabel="Excluir"
        onConfirm={() => handleDelete(deleteId!)}
      >
        Deseja remover este professor? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
