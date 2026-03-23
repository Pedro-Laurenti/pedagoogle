"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Turma, ToastState } from "@/types";

interface TurmaForm extends Record<string, unknown> {
  nome: string;
  ano_letivo: string;
  turno: string;
}

const EMPTY: TurmaForm = { nome: "", ano_letivo: "2026", turno: "Manhã" };

const TURNOS = ["Manhã", "Tarde", "Noite"];

export default function TurmasPage() {
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [form, setForm] = useState<TurmaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    setTurmas(await invokeCmd<Turma[]>("list_turmas"));
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(msg: string, type: ToastState["type"] = "success") {
    setToast({ message: msg, type });
    setTimeout(() => setToast(null), 3000);
  }

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setModal(true);
  }

  function openEdit(t: Turma) {
    setEditing(t.id);
    setForm({ nome: t.nome, ano_letivo: t.ano_letivo, turno: t.turno });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_turma", { id: editing, nome: form.nome, anoLetivo: form.ano_letivo, turno: form.turno });
        notify("Turma atualizada.");
      } else {
        await invokeCmd("create_turma", { nome: form.nome, anoLetivo: form.ano_letivo, turno: form.turno });
        notify("Turma criada.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_turma", { id });
      notify("Turma removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Turmas</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Nova Turma
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Nome</th>
              <th>Ano Letivo</th>
              <th>Turno</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {turmas.map((t) => (
              <tr key={t.id}>
                <td>{t.nome}</td>
                <td>{t.ano_letivo}</td>
                <td>{t.turno}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(t)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(t.id)}>
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
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Nova"} Turma</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Nome</legend>
                <input
                  className="input w-full"
                  value={form.nome}
                  onChange={(e) => setForm({ ...form, nome: e.target.value })}
                  required
                />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Ano Letivo</legend>
                <input
                  className="input w-full"
                  value={form.ano_letivo}
                  onChange={(e) => setForm({ ...form, ano_letivo: e.target.value })}
                  required
                />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Turno</legend>
                <select
                  className="select w-full"
                  value={form.turno}
                  onChange={(e) => setForm({ ...form, turno: e.target.value })}
                >
                  {TURNOS.map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                </select>
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
