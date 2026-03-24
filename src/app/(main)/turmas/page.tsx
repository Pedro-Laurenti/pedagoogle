"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import InputMultiSelect from "@/components/inputs/InputMultiSelect";
import type { Turma, Materia, Configuracoes, ToastState } from "@/types";

interface TurmaForm extends Record<string, unknown> {
  ano: string;
  turma: string;
  turno: string;
}

const EMPTY: TurmaForm = { ano: "", turma: "", turno: "Manhã" };

const TURNOS = ["Manhã", "Tarde", "Noite"];

export default function TurmasPage() {
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<TurmaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [selMaterias, setSelMaterias] = useState<(string | number)[]>([]);

  const load = useCallback(async () => {
    const [ts, ms, cfg] = await Promise.all([
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setTurmas(ts);
    setMaterias(ms);
    setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(msg: string, type: ToastState["type"] = "success") {
    setToast({ message: msg, type });
    setTimeout(() => setToast(null), 3000);
  }

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setSelMaterias([]);
    setModal(true);
  }

  async function openEdit(t: Turma) {
    setEditing(t.id);
    setForm({ ano: t.ano, turma: t.turma, turno: t.turno });
    const ms = await invokeCmd<number[]>("list_turma_materias", { turmaId: t.id });
    setSelMaterias(ms);
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      let id = editing;
      if (editing !== null) {
        await invokeCmd("update_turma", { id: editing, ano: form.ano, turma: form.turma, turno: form.turno });
        notify("Turma atualizada.");
      } else {
        id = await invokeCmd<number>("create_turma", { ano: form.ano, turma: form.turma, turno: form.turno });
        notify("Turma criada.");
      }
      if (id !== null && config?.usar_turmas) {
        await invokeCmd("set_turma_materias", { turmaId: id, materiaIds: selMaterias });
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
      setDeleteId(null);
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
              <th>Turma</th>
              <th>Turno</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {turmas.map((t) => (
              <tr key={t.id}>
                <td>{t.nome}</td>
                <td>{t.turno}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(t)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => setDeleteId(t.id)}>
                    <MdDelete />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Modal open={modal} onClose={() => setModal(false)} title={`${editing !== null ? "Editar" : "Nova"} Turma`}>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Ano *</legend>
              <input
                className="input w-full"
                placeholder="Ex: 6º Ano"
                value={form.ano}
                onChange={(e) => setForm({ ...form, ano: e.target.value })}
                required
              />
            </fieldset>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Turma *</legend>
              <input
                className="input w-full"
                placeholder="Ex: A"
                value={form.turma}
                onChange={(e) => setForm({ ...form, turma: e.target.value })}
                required
              />
            </fieldset>
          </div>
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
          {config?.usar_turmas && (
            <InputMultiSelect
              label="Matérias"
              options={materias.map(m => ({ value: m.id, label: m.nome }))}
              value={selMaterias}
              onChange={setSelMaterias}
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
        Deseja remover esta turma? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
