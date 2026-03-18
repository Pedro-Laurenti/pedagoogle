"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Aula, Materia, ToastState } from "@/types";

interface AulaForm { materia_id: string; dia_semana: string; hora_inicio: string; hora_fim: string; [k: string]: unknown; }

const DIAS = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"];
const EMPTY: AulaForm = { materia_id: "", dia_semana: "Segunda", hora_inicio: "08:00", hora_fim: "09:00" };

export default function CronogramaPage() {
  const [aulas, setAulas] = useState<Aula[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [form, setForm] = useState<AulaForm>(EMPTY);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    const [a, m] = await Promise.all([invokeCmd<Aula[]>("list_aulas"), invokeCmd<Materia[]>("list_materias")]);
    setAulas(a);
    setMaterias(m);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await invokeCmd("create_aula", { materiaId: form.materia_id || null, diaSemana: form.dia_semana, horaInicio: form.hora_inicio, horaFim: form.hora_fim });
      notify("Aula adicionada.");
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_aula", { id });
      notify("Aula removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Cronograma</h1>
        <button className="btn btn-primary" onClick={() => { setForm(EMPTY); setModal(true); }}><MdAdd size={20} /> Nova Aula</button>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Dia</th>
              <th>Matéria</th>
              <th>Início</th>
              <th>Fim</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {DIAS.flatMap((dia) =>
              aulas
                .filter((a) => a.dia_semana === dia)
                .sort((a, b) => a.hora_inicio.localeCompare(b.hora_inicio))
                .map((a) => (
                  <tr key={a.id}>
                    <td>{a.dia_semana}</td>
                    <td>{materias.find((m) => m.id === a.materia_id)?.nome ?? "-"}</td>
                    <td>{a.hora_inicio}</td>
                    <td>{a.hora_fim}</td>
                    <td>
                      <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(a.id)}><MdDelete /></button>
                    </td>
                  </tr>
                ))
            )}
          </tbody>
        </table>
      </div>

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">Nova Aula</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Matéria</legend>
                <select className="select w-full" value={form.materia_id} onChange={(e) => setForm({ ...form, materia_id: e.target.value })}>
                  <option value="">Selecione</option>
                  {materias.map((m) => <option key={m.id} value={m.id}>{m.nome}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Dia da Semana</legend>
                <select className="select w-full" value={form.dia_semana} onChange={(e) => setForm({ ...form, dia_semana: e.target.value })}>
                  {DIAS.map((d) => <option key={d} value={d}>{d}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Hora de Início</legend>
                <input type="time" className="input w-full" value={form.hora_inicio} onChange={(e) => setForm({ ...form, hora_inicio: e.target.value })} required />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Hora de Fim</legend>
                <input type="time" className="input w-full" value={form.hora_fim} onChange={(e) => setForm({ ...form, hora_fim: e.target.value })} required />
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
