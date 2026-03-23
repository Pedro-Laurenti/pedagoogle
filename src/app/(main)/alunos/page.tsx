"use client";
import { useState, useEffect, useCallback, useRef } from "react";
import { MdAdd, MdEdit, MdDelete, MdUploadFile, MdPerson } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Aluno, Turma, AlunoCsvRow, ToastState } from "@/types";

interface AlunoForm {
  nome: string;
  matricula: string;
  turma_id: number | null;
  foto_path: string;
}

const EMPTY: AlunoForm = { nome: "", matricula: "", turma_id: null, foto_path: "" };

export default function AlunosPage() {
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [form, setForm] = useState<AlunoForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [filtroNome, setFiltroNome] = useState("");
  const [filtroTurma, setFiltroTurma] = useState<number | null>(null);
  const [csvModal, setCsvModal] = useState(false);
  const [csvPreview, setCsvPreview] = useState<AlunoCsvRow[]>([]);
  const fileRef = useRef<HTMLInputElement>(null);

  const load = useCallback(async () => {
    const [data, ts] = await Promise.all([
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Turma[]>("list_turmas"),
    ]);
    setAlunos(data);
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

  function openEdit(a: Aluno) {
    setEditing(a.id);
    setForm({ nome: a.nome, matricula: a.matricula, turma_id: a.turma_id, foto_path: a.foto_path });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_aluno", { id: editing, ...form });
        notify("Aluno atualizado.");
      } else {
        await invokeCmd("create_aluno", form);
        notify("Aluno criado.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_aluno", { id });
      notify("Aluno removido.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (ev) => {
      const content = ev.target?.result as string;
      try {
        const rows = await invokeCmd<AlunoCsvRow[]>("preview_import_alunos_csv", { csvContent: content });
        setCsvPreview(rows);
      } catch {
        notify("Erro ao processar CSV.", "error");
      }
    };
    reader.readAsText(file);
  }

  async function confirmImport() {
    try {
      const count = await invokeCmd<number>("confirm_import_alunos", { rows: csvPreview });
      notify(`${count} aluno(s) importado(s).`);
      setCsvModal(false);
      setCsvPreview([]);
      if (fileRef.current) fileRef.current.value = "";
      load();
    } catch {
      notify("Erro ao importar.", "error");
    }
  }

  function openCsvModal() {
    setCsvPreview([]);
    setCsvModal(true);
  }

  const alunosFiltrados = alunos.filter(a =>
    (!filtroNome || a.nome.toLowerCase().includes(filtroNome.toLowerCase()) || a.matricula.includes(filtroNome)) &&
    (!filtroTurma || a.turma_id === filtroTurma)
  );

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Alunos</h1>
        <div className="flex gap-2">
          <button className="btn btn-outline" onClick={openCsvModal}>
            <MdUploadFile size={20} /> Importar CSV
          </button>
          <button className="btn btn-primary" onClick={openCreate}>
            <MdAdd size={20} /> Novo Aluno
          </button>
        </div>
      </div>

      <div className="mb-4 flex items-center gap-3 flex-wrap">
        <input
          className="input input-sm"
          placeholder="Buscar por nome ou matrícula"
          value={filtroNome}
          onChange={(e) => setFiltroNome(e.target.value)}
        />
        <select
          className="select select-sm"
          value={filtroTurma ?? ""}
          onChange={(e) => setFiltroTurma(e.target.value === "" ? null : Number(e.target.value))}
        >
          <option value="">Todas as turmas</option>
          {turmas.map((t) => (
            <option key={t.id} value={t.id}>{t.nome}</option>
          ))}
        </select>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Foto</th>
              <th>Nome</th>
              <th>Matrícula</th>
              <th>Turma</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {alunosFiltrados.map((a) => (
              <tr key={a.id}>
                <td>
                  {a.foto_path ? (
                    <img src={a.foto_path} className="w-8 h-8 rounded-full object-cover" alt={a.nome} />
                  ) : (
                    <MdPerson size={32} className="text-base-content/40" />
                  )}
                </td>
                <td>{a.nome}</td>
                <td>{a.matricula}</td>
                <td>{a.turma_nome ?? "—"}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(a)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(a.id)}>
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
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Novo"} Aluno</h3>
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
                <legend className="fieldset-legend">Matrícula</legend>
                <input
                  className="input w-full"
                  value={form.matricula}
                  onChange={(e) => setForm({ ...form, matricula: e.target.value })}
                />
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
                <legend className="fieldset-legend">Foto (caminho)</legend>
                <input
                  className="input w-full"
                  value={form.foto_path}
                  onChange={(e) => setForm({ ...form, foto_path: e.target.value })}
                  placeholder="/caminho/para/foto.jpg"
                />
              </fieldset>
              <div className="modal-action">
                <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
                <button type="submit" className="btn btn-primary">Salvar</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {csvModal && (
        <div className="modal modal-open">
          <div className="modal-box max-w-2xl">
            <h3 className="font-bold text-lg mb-4">Importar Alunos via CSV</h3>
            <p className="text-sm text-base-content/70 mb-4">Formato: <code>nome,matricula,turma_id</code> (primera linha ignorada)</p>
            <input
              ref={fileRef}
              type="file"
              accept=".csv"
              className="file-input w-full mb-4"
              onChange={handleFileChange}
            />
            {csvPreview.length > 0 && (
              <div className="overflow-x-auto max-h-64">
                <table className="table table-zebra table-sm w-full">
                  <thead>
                    <tr>
                      <th>Nome</th>
                      <th>Matrícula</th>
                      <th>ID Turma</th>
                    </tr>
                  </thead>
                  <tbody>
                    {csvPreview.map((row, i) => (
                      <tr key={i}>
                        <td>{row.nome}</td>
                        <td>{row.matricula}</td>
                        <td>{row.turma_id ?? "—"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            <div className="modal-action">
              <button type="button" className="btn" onClick={() => { setCsvModal(false); setCsvPreview([]); if (fileRef.current) fileRef.current.value = ""; }}>Cancelar</button>
              <button
                type="button"
                className="btn btn-primary"
                disabled={csvPreview.length === 0}
                onClick={confirmImport}
              >
                Confirmar ({csvPreview.length})
              </button>
            </div>
          </div>
        </div>
      )}

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}


interface AlunoForm extends Record<string, unknown> {
  nome: string;
  matricula: string;
  turma_id: number | null;
}

const EMPTY: AlunoForm = { nome: "", matricula: "", turma_id: null };

export default function AlunosPage() {
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [form, setForm] = useState<AlunoForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [filtroTurma, setFiltroTurma] = useState<number | "">("");

  const load = useCallback(async () => {
    const [data, ts] = await Promise.all([
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Turma[]>("list_turmas"),
    ]);
    setAlunos(data);
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

  function openEdit(a: Aluno) {
    setEditing(a.id);
    setForm({ nome: a.nome, matricula: a.matricula, turma_id: a.turma_id });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editing !== null) {
        await invokeCmd("update_aluno", { id: editing, ...form });
        notify("Aluno atualizado.");
      } else {
        await invokeCmd("create_aluno", form);
        notify("Aluno criado.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_aluno", { id });
      notify("Aluno removido.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  const alunosFiltrados = filtroTurma === ""
    ? alunos
    : alunos.filter((a) => a.turma_id === filtroTurma);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Alunos</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Novo Aluno
        </button>
      </div>

      <div className="mb-4 flex items-center gap-3">
        <label className="text-sm font-medium">Filtrar por turma:</label>
        <select
          className="select select-sm"
          value={filtroTurma}
          onChange={(e) => setFiltroTurma(e.target.value === "" ? "" : Number(e.target.value))}
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
              <th>Nome</th>
              <th>Matrícula</th>
              <th>Turma</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {alunosFiltrados.map((a) => (
              <tr key={a.id}>
                <td>{a.nome}</td>
                <td>{a.matricula}</td>
                <td>{a.turma_nome ?? "—"}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(a)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(a.id)}>
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
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Novo"} Aluno</h3>
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
                <legend className="fieldset-legend">Matrícula</legend>
                <input
                  className="input w-full"
                  value={form.matricula}
                  onChange={(e) => setForm({ ...form, matricula: e.target.value })}
                />
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
