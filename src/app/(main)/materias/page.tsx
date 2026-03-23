"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Materia, Professor, Turma, ToastState } from "@/types";

interface MateriaForm {
  nome: string;
  descricao: string;
  professor_id: number | null;
  turma_id: number | null;
  carga_horaria_semanal: number;
  cor: string;
}

const EMPTY: MateriaForm = { nome: "", descricao: "", professor_id: null, turma_id: null, carga_horaria_semanal: 0, cor: "#6366f1" };

export default function MateriasPage() {
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [professores, setProfessores] = useState<Professor[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [form, setForm] = useState<MateriaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [filtroTurma, setFiltroTurma] = useState<number | null>(null);

  const load = useCallback(async () => {
    const [data, profs, ts] = await Promise.all([
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Professor[]>("list_professores"),
      invokeCmd<Turma[]>("list_turmas"),
    ]);
    setMaterias(data);
    setProfessores(profs);
    setTurmas(ts);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setModal(true);
  }

  function openEdit(m: Materia) {
    setEditing(m.id);
    setForm({ nome: m.nome, descricao: m.descricao, professor_id: m.professor_id, turma_id: m.turma_id, carga_horaria_semanal: m.carga_horaria_semanal, cor: m.cor });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_materia", { id: editing, ...form });
        notify("Matéria atualizada.");
      } else {
        await invokeCmd("create_materia", form as unknown as Record<string, unknown>);
        notify("Matéria criada.");
      }
      setModal(false);
      load();
    } catch (e) {
      console.error("Erro ao salvar matéria:", e);
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_materia", { id });
      notify("Matéria removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  const materiasFiltradas = filtroTurma === null
    ? materias
    : materias.filter((m) => m.turma_id === filtroTurma);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Matérias</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Nova Matéria
        </button>
      </div>

      <div className="mb-4 flex items-center gap-3">
        <label className="text-sm font-medium">Filtrar por turma:</label>
        <select
          className="select select-sm"
          value={filtroTurma ?? ""}
          onChange={(e) => setFiltroTurma(e.target.value === "" ? null : Number(e.target.value))}
        >
          <option value="">Todas</option>
          {turmas.map((t) => (
            <option key={t.id} value={t.id}>{t.nome}</option>
          ))}
        </select>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Cor</th>
              <th>Nome</th>
              <th>Turma</th>
              <th>Professor(a)</th>
              <th>Aulas/sem.</th>
              <th>Descrição</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {materiasFiltradas.map((m) => (
              <tr key={m.id}>
                <td>
                  <span
                    className="inline-block w-5 h-5 rounded"
                    style={{ backgroundColor: m.cor }}
                    title={m.cor}
                  />
                </td>
                <td>{m.nome}</td>
                <td>{m.turma_nome ?? "—"}</td>
                <td>{m.professor_nome ?? "—"}</td>
                <td>{m.carga_horaria_semanal}</td>
                <td>{m.descricao}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(m)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(m.id)}>
                    <MdDelete />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Nova"} Matéria</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Nome</legend>
                <input className="input w-full" value={form.nome} onChange={(e) => setForm({ ...form, nome: e.target.value })} required />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Turma</legend>
                <select
                  className="select w-full"
                  value={form.turma_id ?? ""}
                  onChange={(e) => setForm({ ...form, turma_id: e.target.value === "" ? null : Number(e.target.value) })}
                >
                  <option value="">Nenhuma</option>
                  {turmas.map((t) => (
                    <option key={t.id} value={t.id}>{t.nome}</option>
                  ))}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Professor(a)</legend>
                <select
                  className="select w-full"
                  value={form.professor_id ?? ""}
                  onChange={(e) => setForm({ ...form, professor_id: e.target.value ? Number(e.target.value) : null })}
                >
                  <option value="">Nenhum</option>
                  {professores.map((p) => (
                    <option key={p.id} value={p.id}>{p.nome}</option>
                  ))}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Aulas/semana</legend>
                <input
                  type="number"
                  min="0"
                  className="input w-full"
                  value={form.carga_horaria_semanal}
                  onChange={(e) => setForm({ ...form, carga_horaria_semanal: Number(e.target.value) })}
                />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Cor</legend>
                <input
                  type="color"
                  className="input w-full h-10"
                  value={form.cor}
                  onChange={(e) => setForm({ ...form, cor: e.target.value })}
                />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Descrição</legend>
                <input className="input w-full" value={form.descricao} onChange={(e) => setForm({ ...form, descricao: e.target.value })} />
              </fieldset>
              <div className="modal-action">
                <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
                <button type="submit" className="btn btn-primary">Salvar</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
