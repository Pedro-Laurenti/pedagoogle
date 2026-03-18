"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import ProvaEditor from "./ProvaEditor";
import type { Prova, Materia, ToastState } from "@/types";

export default function ProvasPage() {
  const [provas, setProvas] = useState<Prova[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [editing, setEditing] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    const [p, m] = await Promise.all([invokeCmd<Prova[]>("list_provas"), invokeCmd<Materia[]>("list_materias")]);
    setProvas(p);
    setMaterias(m);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_prova", { id });
      notify("Prova removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  if (creating || editing !== null) {
    return (
      <ProvaEditor
        provaId={editing}
        materias={materias}
        onClose={() => { setCreating(false); setEditing(null); load(); }}
        onNotify={notify}
      />
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Provas</h1>
        <button className="btn btn-primary" onClick={() => setCreating(true)}>
          <MdAdd size={20} /> Nova Prova
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Título</th>
              <th>Matéria</th>
              <th>Data</th>
              <th>Valor</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {provas.map((p) => (
              <tr key={p.id}>
                <td>{p.titulo}</td>
                <td>{materias.find((m) => m.id === p.materia_id)?.nome ?? "-"}</td>
                <td>{p.data}</td>
                <td>{p.valor_total} pt</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => setEditing(p.id)}>
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

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}

