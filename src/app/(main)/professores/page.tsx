"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Professor, Materia, Turma, Configuracoes, ToastState } from "@/types";

interface ProfessorForm {
  nome: string; email: string; telefone: string; especialidade: string;
  aulas_por_semana: number; observacoes: string;
  [k: string]: unknown;
}

const EMPTY: ProfessorForm = { nome: "", email: "", telefone: "", especialidade: "", aulas_por_semana: 0, observacoes: "" };

export default function ProfessoresPage() {
  const [professores, setProfessores] = useState<Professor[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<ProfessorForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  // selected materias/turmas ids
  const [selMaterias, setSelMaterias] = useState<number[]>([]);
  const [selTurmas, setSelTurmas] = useState<number[]>([]);

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
    setForm({ nome: p.nome, email: p.email, telefone: p.telefone ?? "", especialidade: p.especialidade ?? "", aulas_por_semana: p.aulas_por_semana ?? 0, observacoes: p.observacoes ?? "" });
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
        await invokeCmd("update_professor", { id: editing, nome: form.nome, email: form.email, telefone: form.telefone, especialidade: form.especialidade, aulasPorSemana: form.aulas_por_semana, observacoes: form.observacoes });
        notify("Professor atualizado.");
      } else {
        id = await invokeCmd<number>("create_professor", { nome: form.nome, email: form.email, telefone: form.telefone, especialidade: form.especialidade, aulasPorSemana: form.aulas_por_semana, observacoes: form.observacoes });
        notify("Professor criado.");
      }
      if (id !== null) {
        await Promise.all([
          invokeCmd("set_professor_materias", { professorId: id, materiaIds: selMaterias }),
          invokeCmd("set_professor_turmas", { professorId: id, turmaIds: selTurmas }),
        ]);
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
      load();
    } catch (err) {
      notify(String(err), "error");
    }
  }

  function toggleMateria(id: number) {
    setSelMaterias(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]);
  }

  function toggleTurma(id: number) {
    setSelTurmas(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]);
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
              <th>Especialidade</th>
              <th>Matérias</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {professores.map((p) => (
              <tr key={p.id}>
                <td>{p.nome}</td>
                <td>{p.email}</td>
                <td>{p.especialidade || "–"}</td>
                <td className="text-sm text-base-content/60">{p.especialidade ? p.especialidade : "–"}</td>
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
          <div className="modal-box max-w-2xl">
            <h3 className="font-bold text-lg mb-4">{editing !== null ? "Editar" : "Novo"} Professor</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Nome *</legend>
                  <input className="input w-full" value={form.nome} onChange={(e) => setForm({ ...form, nome: e.target.value })} required />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">E-mail</legend>
                  <input className="input w-full" type="email" value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Telefone</legend>
                  <input className="input w-full" value={form.telefone} onChange={(e) => setForm({ ...form, telefone: e.target.value })} placeholder="(00) 00000-0000" />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Especialidade</legend>
                  <input className="input w-full" value={form.especialidade} onChange={(e) => setForm({ ...form, especialidade: e.target.value })} placeholder="Ex: Matemática" />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Aulas por semana</legend>
                  <input type="number" className="input w-full" min={0} value={form.aulas_por_semana} onChange={(e) => setForm({ ...form, aulas_por_semana: parseInt(e.target.value) || 0 })} />
                </fieldset>
              </div>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Observações</legend>
                <textarea className="textarea w-full" rows={2} value={form.observacoes} onChange={(e) => setForm({ ...form, observacoes: e.target.value })} />
              </fieldset>

              {materias.length > 0 && (
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Matérias que leciona</legend>
                  <div className="flex flex-wrap gap-2">
                    {materias.map(m => (
                      <label key={m.id} className={`flex items-center gap-1.5 cursor-pointer px-3 py-1.5 rounded-full border text-sm transition-colors ${selMaterias.includes(m.id) ? "border-primary bg-primary/10 text-primary" : "border-base-300 hover:border-base-content/30"}`}>
                        <input type="checkbox" className="sr-only" checked={selMaterias.includes(m.id)} onChange={() => toggleMateria(m.id)} />
                        <span className="w-2 h-2 rounded-full" style={{ backgroundColor: m.cor ?? "#6366f1" }} />
                        {m.nome}
                      </label>
                    ))}
                  </div>
                </fieldset>
              )}

              {config?.usar_turmas && turmas.length > 0 && (
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Turmas</legend>
                  <div className="flex flex-wrap gap-2">
                    {turmas.map(t => (
                      <label key={t.id} className={`flex items-center gap-1.5 cursor-pointer px-3 py-1.5 rounded-full border text-sm transition-colors ${selTurmas.includes(t.id) ? "border-secondary bg-secondary/10 text-secondary" : "border-base-300 hover:border-base-content/30"}`}>
                        <input type="checkbox" className="sr-only" checked={selTurmas.includes(t.id)} onChange={() => toggleTurma(t.id)} />
                        {t.nome}
                      </label>
                    ))}
                  </div>
                </fieldset>
              )}

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
