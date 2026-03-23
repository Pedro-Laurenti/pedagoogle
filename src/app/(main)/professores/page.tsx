"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Professor, ToastState } from "@/types";

interface ProfessorForm { nome: string; email: string; [k: string]: unknown; }

const EMPTY: ProfessorForm = { nome: "", email: "" };

export default function ProfessoresPage() {
  const [professores, setProfessores] = useState<Professor[]>([]);
  const [form, setForm] = useState<ProfessorForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    setProfessores(await invokeCmd<Professor[]>("list_professores"));
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

  function openEdit(p: Professor) {
    setEditing(p.id);
    setForm({ nome: p.nome, email: p.email });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_professor", { id: editing, ...form });
        notify("Professor atualizado.");
      } else {
        await invokeCmd("create_professor", form);
        notify("Professor criado.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_professor", { id });
      notify("Professor removido.");
      load();
    } catch (err) {
      notify(String(err), "error");
    }
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
              <th>E-mail</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {professores.map((p) => (
              <tr key={p.id}>
                <td>{p.nome}</td>
                <td>{p.email}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(p)}>
                    <MdEdit />
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

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Novo"} Professor</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Nome</legend>
                <input className="input w-full" value={form.nome} onChange={(e) => setForm({ ...form, nome: e.target.value })} required />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">E-mail</legend>
                <input className="input w-full" type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} />
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
