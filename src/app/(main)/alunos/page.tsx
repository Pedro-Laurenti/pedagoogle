"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdPerson } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import { convertFileSrc } from "@tauri-apps/api/core";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import Pagination from "@/components/Pagination";
import InputImagem from "@/components/inputs/InputImagem";
import InputMultiSelect from "@/components/inputs/InputMultiSelect";
import type { Aluno, Turma, Materia, Configuracoes, ToastState } from "@/types";

const PER_PAGE = 20;

interface AlunoForm {
  nome: string;
  turma_id: number | null;
  foto_path: string;
}

const EMPTY: AlunoForm = { nome: "", turma_id: null, foto_path: "" };

export default function AlunosPage() {
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<AlunoForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [filtroNome, setFiltroNome] = useState("");
  const [filtroTurma, setFiltroTurma] = useState<number | null>(null);
  const [selMaterias, setSelMaterias] = useState<(string | number)[]>([]);
  const [page, setPage] = useState(1);

  const load = useCallback(async () => {
    const [data, ts, ms, cfg] = await Promise.all([
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setAlunos(data);
    setTurmas(ts);
    setMaterias(ms);
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
    setModal(true);
  }

  async function openEdit(a: Aluno) {
    setEditing(a.id);
    setForm({ nome: a.nome, turma_id: a.turma_id, foto_path: a.foto_path });
    const ms = await invokeCmd<number[]>("list_aluno_materias", { alunoId: a.id });
    setSelMaterias(ms);
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      let id = editing;
      if (editing !== null) {
        await invokeCmd("update_aluno", { id: editing, nome: form.nome, turmaId: form.turma_id, fotoPath: form.foto_path });
        notify("Aluno atualizado.");
      } else {
        id = await invokeCmd<number>("create_aluno", { nome: form.nome, turmaId: form.turma_id, fotoPath: form.foto_path });
        notify("Aluno criado.");
      }
      if (id !== null && !config?.usar_turmas) {
        await invokeCmd("set_aluno_materias", { alunoId: id, materiaIds: selMaterias });
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
      setDeleteId(null);
      load();
    } catch (err) {
      notify(String(err), "error");
    }
  }

  const alunosFiltrados = alunos.filter(a =>
    (!filtroNome || a.nome.toLowerCase().includes(filtroNome.toLowerCase())) &&
    (!filtroTurma || a.turma_id === filtroTurma)
  );
  const alunosPagina = alunosFiltrados.slice((page - 1) * PER_PAGE, page * PER_PAGE);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Alunos</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Novo Aluno
        </button>
      </div>

      <div className="mb-4 flex items-center gap-3 flex-wrap">
        <input
          className="input input-sm"
          placeholder="Buscar por nome"
          value={filtroNome}
          onChange={(e) => { setFiltroNome(e.target.value); setPage(1); }}
        />
        {config?.usar_turmas && (
          <select
            className="select select-sm"
            value={filtroTurma ?? ""}
            onChange={(e) => { setFiltroTurma(e.target.value === "" ? null : Number(e.target.value)); setPage(1); }}
          >
            <option value="">Todas as turmas</option>
            {turmas.map((t) => (
              <option key={t.id} value={t.id}>{t.nome}</option>
            ))}
          </select>
        )}
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Foto</th>
              <th>Nome</th>
              {config?.usar_turmas && <th>Turma</th>}
              <th></th>
            </tr>
          </thead>
          <tbody>
            {alunosPagina.map((a) => (
              <tr key={a.id}>
                <td>
                  {a.foto_path ? (
                    <img src={convertFileSrc(a.foto_path)} className="w-8 h-8 rounded-full object-cover" alt={a.nome} onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }} />
                  ) : (
                    <MdPerson size={32} className="text-base-content/40" />
                  )}
                </td>
                <td>{a.nome}</td>
                {config?.usar_turmas && <td>{a.turma_nome ?? "—"}</td>}
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(a)}>
                    <MdEdit />
                  </button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => setDeleteId(a.id)}>
                    <MdDelete />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <Pagination page={page} total={alunosFiltrados.length} perPage={PER_PAGE} onChange={setPage} />

      <Modal open={modal} onClose={() => setModal(false)} title={`${editing !== null ? "Editar" : "Novo"} Aluno`}>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Nome *</legend>
            <input
              className="input w-full"
              value={form.nome}
              onChange={(e) => setForm({ ...form, nome: e.target.value })}
              required
            />
          </fieldset>
          {config?.usar_turmas ? (
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
          ) : (
            <InputMultiSelect
              label="Matérias *"
              options={materias.map(m => ({ value: m.id, label: m.nome }))}
              value={selMaterias}
              onChange={setSelMaterias}
            />
          )}
          <InputImagem
            label="Foto"
            value={form.foto_path}
            onChange={(path) => setForm({ ...form, foto_path: path })}
          />
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
        Deseja remover este aluno? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}