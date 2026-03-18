"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Materia, ToastState } from "@/types";

interface MateriaForm { nome: string; descricao: string; professor: string; [k: string]: unknown; }

const EMPTY: MateriaForm = { nome: "", descricao: "", professor: "" };

export default function MateriasPage() {
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [form, setForm] = useState<MateriaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    const data = await invokeCmd<Materia[]>("list_materias");
    setMaterias(data);
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
    setForm({ nome: m.nome, descricao: m.descricao, professor: m.professor });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_materia", { id: editing, ...form });
        notify("Matéria atualizada.");
      } else {
        await invokeCmd("create_materia", form);
        notify("Matéria criada.");
      }
      setModal(false);
      load();
    } catch {
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

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Matérias</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Nova Matéria
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Nome</th>
              <th>Professor(a)</th>
              <th>Descrição</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {materias.map((m) => (
              <tr key={m.id}>
                <td>{m.nome}</td>
                <td>{m.professor}</td>
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
                <legend className="fieldset-legend">Professor(a)</legend>
                <input className="input w-full" value={form.professor} onChange={(e) => setForm({ ...form, professor: e.target.value })} />
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
