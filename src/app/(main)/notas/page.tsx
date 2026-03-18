"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Nota, Aluno, Prova, ToastState } from "@/types";

interface NotaForm { aluno_id: string; prova_id: string; descricao: string; valor: string; [k: string]: unknown; }

const EMPTY: NotaForm = { aluno_id: "", prova_id: "", descricao: "", valor: "" };

export default function NotasPage() {
  const [notas, setNotas] = useState<Nota[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [provas, setProvas] = useState<Prova[]>([]);
  const [form, setForm] = useState<NotaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [filterAluno, setFilterAluno] = useState("");
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    const [n, a, p] = await Promise.all([
      invokeCmd<Nota[]>("list_notas"),
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Prova[]>("list_provas"),
    ]);
    setNotas(n);
    setAlunos(a);
    setProvas(p);
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

  function openEdit(n: Nota) {
    setEditing(n.id);
    setForm({ aluno_id: String(n.aluno_id), prova_id: n.prova_id ? String(n.prova_id) : "", descricao: n.descricao, valor: String(n.valor) });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const payload = {
      alunoId: Number(form.aluno_id),
      provaId: form.prova_id ? Number(form.prova_id) : null,
      descricao: form.descricao,
      valor: parseFloat(form.valor),
    };
    try {
      if (editing !== null) {
        await invokeCmd("update_nota", { id: editing, ...payload });
        notify("Nota atualizada.");
      } else {
        await invokeCmd("create_nota", payload);
        notify("Nota lançada.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_nota", { id });
      notify("Nota removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  const filtered = filterAluno ? notas.filter((n) => String(n.aluno_id) === filterAluno) : notas;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Notas</h1>
        <button className="btn btn-primary" onClick={openCreate}><MdAdd size={20} /> Lançar Nota</button>
      </div>

      <div className="mb-4">
        <select className="select w-64" value={filterAluno} onChange={(e) => setFilterAluno(e.target.value)}>
          <option value="">Todos os alunos</option>
          {alunos.map((a) => <option key={a.id} value={a.id}>{a.nome}</option>)}
        </select>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Aluno</th>
              <th>Prova / Atividade</th>
              <th>Descrição</th>
              <th>Valor</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((n) => (
              <tr key={n.id}>
                <td>{alunos.find((a) => a.id === n.aluno_id)?.nome ?? "-"}</td>
                <td>{provas.find((p) => p.id === n.prova_id)?.titulo ?? "-"}</td>
                <td>{n.descricao}</td>
                <td>{n.valor}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(n)}><MdEdit /></button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(n.id)}><MdDelete /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Lançar"} Nota</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Aluno</legend>
                <select className="select w-full" value={form.aluno_id} onChange={(e) => setForm({ ...form, aluno_id: e.target.value })} required>
                  <option value="">Selecione</option>
                  {alunos.map((a) => <option key={a.id} value={a.id}>{a.nome}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Prova (opcional)</legend>
                <select className="select w-full" value={form.prova_id} onChange={(e) => setForm({ ...form, prova_id: e.target.value })}>
                  <option value="">Nenhuma</option>
                  {provas.map((p) => <option key={p.id} value={p.id}>{p.titulo}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Descrição</legend>
                <input className="input w-full" value={form.descricao} onChange={(e) => setForm({ ...form, descricao: e.target.value })} placeholder="Ex: Trabalho, Simulado..." />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Valor</legend>
                <input type="number" step="0.1" className="input w-full" value={form.valor} onChange={(e) => setForm({ ...form, valor: e.target.value })} required />
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
