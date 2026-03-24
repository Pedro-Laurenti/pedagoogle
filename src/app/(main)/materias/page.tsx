"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdArrowForward, MdArrowBack } from "react-icons/md";
import * as MdIcons from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import ColorPicker from "@/components/ColorPicker";
import IconPicker from "@/components/IconPicker";
import { InputTexto } from "@/components/inputs";
import type { Materia, Professor, Turma, Configuracoes, ToastState } from "@/types";

const MATERIA_ICONS = [
  'MdBook', 'MdScience', 'MdCalculate', 'MdLanguage', 'MdHistoryEdu',
  'MdSportsFootball', 'MdMusicNote', 'MdPalette', 'MdComputer', 'MdBiotech',
  'MdPublic', 'MdFunctions', 'MdMenuBook', 'MdSchool', 'MdStar',
];

interface MateriaForm {
  nome: string;
  professor_id: number | null;
  turma_id: number | null;
  carga_horaria_semanal: number;
  cor: string;
  icone: string;
}

const EMPTY: MateriaForm = { nome: "", professor_id: null, turma_id: null, carga_horaria_semanal: 0, cor: "#6366f1", icone: "MdBook" };

export default function MateriasPage() {
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [professores, setProfessores] = useState<Professor[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<MateriaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [step, setStep] = useState<1 | 2>(1);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [filtroTurma, setFiltroTurma] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);

  const load = useCallback(async () => {
    const [data, profs, ts, cfg] = await Promise.all([
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Professor[]>("list_professores"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setMaterias(data);
    setProfessores(profs);
    setTurmas(ts);
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
    setStep(1);
    setModal(true);
  }

  function openEdit(m: Materia) {
    setEditing(m.id);
    setForm({ nome: m.nome, professor_id: m.professor_id, turma_id: m.turma_id, carga_horaria_semanal: m.carga_horaria_semanal, cor: m.cor, icone: m.icone ?? 'MdBook' });
    setStep(1);
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const payload = {
        nome: form.nome,
        professorId: form.professor_id,
        turmaId: form.turma_id,
        cargaHorariaSemanal: form.carga_horaria_semanal,
        cor: form.cor,
        icone: form.icone,
      };
      if (editing !== null) {
        await invokeCmd("update_materia", { id: editing, ...payload });
        notify("Matéria atualizada.");
      } else {
        await invokeCmd("create_materia", payload);
        notify("Matéria criada.");
      }
      setModal(false);
      load();
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function confirmDelete() {
    if (deleteId === null) return;
    try {
      await invokeCmd("delete_materia", { id: deleteId });
      notify("Matéria removida.");
      load();
    } catch (e: unknown) {
      notify(e instanceof Error ? e.message : String(e), "error");
    } finally {
      setDeleteId(null);
    }
  }

  const hasStep2 = (config?.usar_professores || config?.usar_turmas) ?? false;
  const diasSemana = config?.dias_letivos_semana ?? 5;
  const aulasMaxDia = config?.aulas_por_dia ?? 6;
  const aulasSemana = form.carga_horaria_semanal;
  const livresSemana = aulasMaxDia * diasSemana - aulasSemana;
  const livresDia = diasSemana > 0 ? aulasMaxDia - aulasSemana / diasSemana : 0;

  const materiasFiltradas = config?.usar_turmas && filtroTurma !== null
    ? materias.filter((m) => m.turma_id === filtroTurma)
    : materias;

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold">Matérias</h1>
        <button className="btn btn-primary" onClick={openCreate}>
          <MdAdd size={20} /> Nova Matéria
        </button>
      </div>

      {config?.usar_turmas && (
        <div className="mb-4 flex items-center gap-3">
          <label className="text-sm font-medium">Filtrar por turma:</label>
          <select
            className="select select-sm"
            value={filtroTurma ?? ""}
            onChange={(e) => setFiltroTurma(e.target.value === "" ? null : Number(e.target.value))}
          >
            <option value="">Todas</option>
            {turmas.map((t) => (
              <option key={t.id} value={t.id}>{t.nome}</option>
            ))}
          </select>
        </div>
      )}

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Cor</th>
              <th>Nome</th>
              {config?.usar_turmas && <th>Turma</th>}
              {config?.usar_professores && <th>Professor(a)</th>}
              <th>Aulas/sem.</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {materiasFiltradas.map((m) => {
              const Icon = (MdIcons as Record<string, React.ElementType>)[m.icone ?? 'MdBook'];
              return (
                <tr key={m.id}>
                  <td>
                    <span className="inline-flex w-7 h-7 rounded items-center justify-center" style={{ backgroundColor: m.cor }}>
                      {Icon && <Icon size={16} className="text-white" />}
                    </span>
                  </td>
                  <td>{m.nome}</td>
                  {config?.usar_turmas && <td>{m.turma_nome ?? "—"}</td>}
                  {config?.usar_professores && <td>{m.professor_nome ?? "—"}</td>}
                  <td>{m.carga_horaria_semanal}</td>
                  <td className="flex gap-2">
                    <button className="btn btn-sm btn-ghost" onClick={() => openEdit(m)}>
                      <MdEdit />
                    </button>
                    <button className="btn btn-sm btn-ghost text-error" onClick={() => setDeleteId(m.id)}>
                      <MdDelete />
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <Modal open={modal} onClose={() => setModal(false)} title={`${editing !== null ? "Editar" : "Nova"} Matéria`} size="lg">
        <form onSubmit={handleSubmit}>
          {step === 1 && (
            <div className="flex flex-col gap-4">
              <InputTexto label="Nome" value={form.nome} onChange={(v) => setForm({ ...form, nome: v })} required />
              <div className="flex flex-col gap-2">
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Aulas/semana</legend>
                  <input
                    type="number"
                    min="0"
                    max={aulasMaxDia * diasSemana}
                    className="input w-full"
                    value={aulasSemana}
                    onChange={(e) => setForm({ ...form, carga_horaria_semanal: Number(e.target.value) })}
                  />
                </fieldset>
                {aulasSemana > 0 && (
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Livres/dia</div>
                      <div className="stat-value text-lg">{livresDia.toFixed(1)}</div>
                    </div>
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Livres/semana</div>
                      <div className="stat-value text-lg">{livresSemana}</div>
                    </div>
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Aulas/mês</div>
                      <div className="stat-value text-lg">{aulasSemana * 4}</div>
                    </div>
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Aulas/bimestre</div>
                      <div className="stat-value text-lg">{aulasSemana * 8}</div>
                    </div>
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Aulas/semestre</div>
                      <div className="stat-value text-lg">{aulasSemana * 20}</div>
                    </div>
                    <div className="stat bg-base-200 rounded-box p-3">
                      <div className="stat-title text-xs">Aulas/ano</div>
                      <div className="stat-value text-lg">{aulasSemana * 40}</div>
                    </div>
                  </div>
                )}
              </div>
              <ColorPicker value={form.cor} onChange={(c) => setForm({ ...form, cor: c })} label="Cor" />
              <IconPicker value={form.icone} onChange={(i) => setForm({ ...form, icone: i })} label="Ícone" icons={MATERIA_ICONS} iconLib="md" />
              <div className="modal-action">
                <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
                {hasStep2
                  ? <button type="button" className="btn btn-primary" onClick={() => setStep(2)}><MdArrowForward /> Próximo</button>
                  : <button type="submit" className="btn btn-primary">Salvar</button>
                }
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="flex flex-col gap-4">
              {config?.usar_professores && (
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Professor(a)</legend>
                  <select
                    className="select w-full"
                    value={form.professor_id ?? ""}
                    onChange={(e) => setForm({ ...form, professor_id: e.target.value ? Number(e.target.value) : null })}
                  >
                    <option value="">Nenhum</option>
                    {professores.map((p) => (
                      <option key={p.id} value={p.id}>{p.nome}</option>
                    ))}
                  </select>
                </fieldset>
              )}
              {config?.usar_turmas && (
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
              )}
              <div className="modal-action">
                <button type="button" className="btn" onClick={() => setStep(1)}><MdArrowBack /> Voltar</button>
                <button type="submit" className="btn btn-primary">Salvar</button>
              </div>
            </div>
          )}
        </form>
      </Modal>

      <Modal
        open={deleteId !== null}
        onClose={() => setDeleteId(null)}
        variant="confirm"
        color="error"
        title="Excluir Matéria"
        confirmLabel="Excluir"
        onConfirm={confirmDelete}
      >
        Tem certeza que deseja excluir esta matéria? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
