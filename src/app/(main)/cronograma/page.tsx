"use client";
import { useState, useEffect, useCallback, useMemo } from "react";
import { MdAdd, MdEdit, MdDelete, MdCopyAll, MdViewModule, MdViewList, MdEventNote } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import { InputHora, InputMultiSelect } from "@/components/inputs";
import type { Aula, Materia, Turma, Aluno, Configuracoes, ToastState, FaltaItem } from "@/types";

interface AulaForm {
  materia_id: string;
  dias_recorrencia: string[];
  hora_inicio: string;
  hora_fim: string;
  turma_id: string;
  aluno_ids: string;
}

const DIAS_SEMANA = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"];
const DIAS_ABREV: Record<string, string> = { "Segunda": "Seg", "Terça": "Ter", "Quarta": "Qua", "Quinta": "Qui", "Sexta": "Sex", "Sábado": "Sáb" };
const SEMESTRE_PADRAO = `${new Date().getFullYear()}-1`;

function minsToTime(mins: number): string {
  return `${String(Math.floor(mins / 60)).padStart(2, "0")}:${String(mins % 60).padStart(2, "0")}`;
}

function timeToMins(t: string): number {
  const [h, m] = t.split(":").map(Number);
  return h * 60 + m;
}

function getSlots(config: Configuracoes) {
  const start = timeToMins(config.hora_entrada);
  return Array.from({ length: config.aulas_por_dia }, (_, i) => ({
    inicio: minsToTime(start + i * config.minutos_por_aula),
    fim: minsToTime(start + (i + 1) * config.minutos_por_aula),
  }));
}

function emptyForm(config: Configuracoes | null, dia = "Segunda"): AulaForm {
  const horaInicio = config?.hora_entrada ?? "07:00";
  const horaFim = config ? minsToTime(timeToMins(horaInicio) + config.minutos_por_aula) : "07:50";
  return { materia_id: "", dias_recorrencia: [dia], hora_inicio: horaInicio, hora_fim: horaFim, turma_id: "", aluno_ids: "[]" };
}

export default function CronogramaPage() {
  const [aulas, setAulas] = useState<Aula[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<AulaForm>(emptyForm(null));
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [deleteId, setDeleteId] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [modoGrade, setModoGrade] = useState(true);
  const [semestreAtual, setSemestreAtual] = useState(SEMESTRE_PADRAO);
  const [showNovoSem, setShowNovoSem] = useState(false);
  const [novoSemInput, setNovoSemInput] = useState("");
  const [copiarModal, setCopiarModal] = useState(false);
  const [semestreDestino, setSemestreDestino] = useState("");
  const [copiando, setCopiando] = useState(false);
  const [presencaAulaId, setPresencaAulaId] = useState<number | null>(null);
  const [presencaData, setPresencaData] = useState(new Date().toISOString().split("T")[0]);
  const [presencaItems, setPresencaItems] = useState<FaltaItem[]>([]);
  const [savingPresenca, setSavingPresenca] = useState(false);

  const load = useCallback(async () => {
    const [a, m, t, al, cfg] = await Promise.all([
      invokeCmd<Aula[]>("list_aulas", { semestre: null }),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setAulas(a); setMaterias(m); setTurmas(t); setAlunos(al); setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);

  const semestres = [...new Set([...aulas.map(a => a.semestre), semestreAtual])].sort();
  const aulasFiltradas = aulas.filter(a => a.semestre === semestreAtual);
  const opcoesDias = useMemo(
    () => DIAS_SEMANA.slice(0, config?.dias_letivos_semana ?? 5).map(d => ({ value: d, label: d })),
    [config]
  );

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  function openEdit(a: Aula) {
    setEditing(a.id);
    setForm({ materia_id: a.materia_id ? String(a.materia_id) : "", dias_recorrencia: [a.dia_semana], hora_inicio: a.hora_inicio, hora_fim: a.hora_fim, turma_id: a.turma_id ? String(a.turma_id) : "", aluno_ids: a.aluno_ids ?? "[]" });
    setModal(true);
  }

  function openFromCell(dia: string, horaInicio: string, horaFim: string) {
    setEditing(null);
    setForm({ ...emptyForm(config, dia), hora_inicio: horaInicio, hora_fim: horaFim, dias_recorrencia: [dia] });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const base = { materiaId: form.materia_id ? Number(form.materia_id) : null, horaInicio: form.hora_inicio, horaFim: form.hora_fim, semestre: semestreAtual, turmaId: form.turma_id ? Number(form.turma_id) : null, alunoIds: form.aluno_ids || "[]" };
    try {
      if (editing !== null) {
        await invokeCmd("update_aula", { id: editing, diaSemana: form.dias_recorrencia[0] ?? "Segunda", ...base });
        notify("Aula atualizada.");
      } else if (form.dias_recorrencia.length > 1) {
        const qtd = await invokeCmd<number>("create_aulas_recorrentes", { diasSemana: form.dias_recorrencia, ...base });
        notify(`${qtd} aula(s) adicionada(s).`);
      } else {
        await invokeCmd("create_aula", { diaSemana: form.dias_recorrencia[0] ?? "Segunda", ...base });
        notify("Aula adicionada.");
      }
      setModal(false);
      load();
    } catch (err) {
      notify(String(err) || "Erro ao salvar.", "error");
    }
  }

  async function handleDelete() {
    if (deleteId === null) return;
    try {
      await invokeCmd("delete_aula", { id: deleteId });
      notify("Aula removida.");
      setDeleteId(null);
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  async function handleCopiar() {
    if (!semestreDestino.trim()) return;
    setCopiando(true);
    try {
      const qtd = await invokeCmd<number>("copy_semestre", { de: semestreAtual, para: semestreDestino.trim() });
      notify(`${qtd} aulas copiadas para ${semestreDestino.trim()}.`);
      setSemestreAtual(semestreDestino.trim());
      setCopiarModal(false);
      setSemestreDestino("");
      load();
    } catch (err) {
      notify(String(err) || "Erro ao copiar.", "error");
    } finally {
      setCopiando(false);
    }
  }

  async function openPresenca(aulaId: number) {
    setPresencaAulaId(aulaId);
    const items = await invokeCmd<FaltaItem[]>("get_faltas_aula", { aulaId, data: presencaData });
    setPresencaItems(items);
  }

  async function savePresenca() {
    if (presencaAulaId === null) return;
    setSavingPresenca(true);
    const faltaram = presencaItems.filter(i => i.faltou).map(i => i.aluno_id);
    try {
      await invokeCmd("save_faltas_aula", { aulaId: presencaAulaId, data: presencaData, faltaram });
      notify("Frequência salva.");
      setPresencaAulaId(null);
    } catch (e) {
      notify(String(e), "error");
    } finally {
      setSavingPresenca(false);
    }
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-4 flex-wrap gap-2">
        <h1 className="text-3xl font-bold">Cronograma</h1>
        <div className="flex items-center gap-2 flex-wrap">
          {!showNovoSem ? (
            <select className="select select-sm" value={semestreAtual} onChange={e => {
              if (e.target.value === "__novo__") { setShowNovoSem(true); setNovoSemInput(""); }
              else setSemestreAtual(e.target.value);
            }}>
              {semestres.map(s => <option key={s} value={s}>{s}</option>)}
              <option value="__novo__">+ Novo</option>
            </select>
          ) : (
            <div className="flex gap-1">
              <input className="input input-sm w-28" value={novoSemInput} onChange={e => setNovoSemInput(e.target.value)} placeholder="ex: 2026-2" autoFocus />
              <button className="btn btn-sm btn-primary" onClick={() => { if (novoSemInput.trim()) setSemestreAtual(novoSemInput.trim()); setShowNovoSem(false); }}>OK</button>
              <button className="btn btn-sm" onClick={() => setShowNovoSem(false)}>×</button>
            </div>
          )}
          <button className="btn btn-sm btn-ghost" title="Copiar para novo semestre" onClick={() => { setSemestreDestino(""); setCopiarModal(true); }}><MdCopyAll size={18} /></button>
          <div className="join">
            <button className={`btn btn-sm join-item ${modoGrade ? "btn-active" : ""}`} onClick={() => setModoGrade(true)} title="Grade"><MdViewModule size={18} /></button>
            <button className={`btn btn-sm join-item ${!modoGrade ? "btn-active" : ""}`} onClick={() => setModoGrade(false)} title="Lista"><MdViewList size={18} /></button>
          </div>
          <button className="btn btn-primary btn-sm" onClick={() => { setEditing(null); setForm(emptyForm(config)); setModal(true); }}><MdAdd size={18} /> Nova Aula</button>
        </div>
      </div>

      {modoGrade ? (
        <GradeView aulas={aulasFiltradas} materias={materias} config={config} onEdit={openEdit} onCellClick={openFromCell} onFrequencia={openPresenca} />
      ) : (
        <ListaView aulas={aulasFiltradas} materias={materias} onEdit={openEdit} onDeleteRequest={id => setDeleteId(id)} />
      )}

      <Modal open={modal} onClose={() => setModal(false)} title={editing !== null ? "Editar Aula" : "Nova Aula"}>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Matéria</legend>
            <select className="select w-full" value={form.materia_id} onChange={e => setForm({ ...form, materia_id: e.target.value })}>
              <option value="">Selecione</option>
              {materias.map(m => <option key={m.id} value={m.id}>{m.nome}</option>)}
            </select>
          </fieldset>
          <InputMultiSelect
            label={editing !== null ? "Dia da semana" : "Dias da semana (recorrência)"}
            options={opcoesDias}
            value={form.dias_recorrencia}
            onChange={v => setForm({ ...form, dias_recorrencia: v as string[] })}
          />
          <div className="grid grid-cols-2 gap-3">
            <InputHora label="Início" value={form.hora_inicio} onChange={v => setForm({ ...form, hora_inicio: v })} required />
            <InputHora label="Fim" value={form.hora_fim} onChange={v => setForm({ ...form, hora_fim: v })} required />
          </div>
          {config?.usar_turmas && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Turma (opcional)</legend>
              <select className="select w-full" value={form.turma_id} onChange={e => setForm({ ...form, turma_id: e.target.value })}>
                <option value="">Nenhuma</option>
                {turmas.map(t => <option key={t.id} value={t.id}>{t.nome}</option>)}
              </select>
            </fieldset>
          )}
          {alunos.length > 0 && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Alunos (opcional)</legend>
              <div className="max-h-40 overflow-y-auto border border-base-300 rounded p-2 flex flex-col gap-1">
                {alunos.map(al => {
                  const ids: number[] = JSON.parse(form.aluno_ids || "[]");
                  const checked = ids.includes(al.id);
                  return (
                    <label key={al.id} className="flex items-center gap-2 cursor-pointer">
                      <input type="checkbox" className="checkbox checkbox-sm" checked={checked} onChange={() => {
                        const newIds = checked ? ids.filter(i => i !== al.id) : [...ids, al.id];
                        setForm({ ...form, aluno_ids: JSON.stringify(newIds) });
                      }} />
                      <span className="text-sm">{al.nome}</span>
                    </label>
                  );
                })}
              </div>
            </fieldset>
          )}
          <div className="modal-action">
            <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
            <button type="submit" className="btn btn-primary">Salvar</button>
          </div>
        </form>
      </Modal>

      <Modal open={deleteId !== null} onClose={() => setDeleteId(null)} title="Confirmar exclusão" variant="confirm" color="error" confirmLabel="Excluir" onConfirm={handleDelete}>
        Deseja excluir esta aula? Esta ação não pode ser desfeita.
      </Modal>

      <Modal open={copiarModal} onClose={() => setCopiarModal(false)} title="Copiar Semestre">
        <p className="text-sm mb-4 text-base-content/70">Copiar todas as aulas de <strong>{semestreAtual}</strong> para:</p>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Semestre destino</legend>
          <input className="input w-full" value={semestreDestino} onChange={e => setSemestreDestino(e.target.value)} placeholder="ex: 2026-2" autoFocus />
        </fieldset>
        <div className="modal-action">
          <button className="btn" onClick={() => setCopiarModal(false)}>Cancelar</button>
          <button className="btn btn-primary" onClick={handleCopiar} disabled={copiando || !semestreDestino.trim()}>{copiando ? "Copiando..." : "Copiar"}</button>
        </div>
      </Modal>

      <Modal open={presencaAulaId !== null} onClose={() => setPresencaAulaId(null)} title="Lançar Frequência" size="md">
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Data da aula</legend>
          <input type="date" className="input w-full" value={presencaData} onChange={async e => {
            setPresencaData(e.target.value);
            if (presencaAulaId !== null) {
              const items = await invokeCmd<FaltaItem[]>("get_faltas_aula", { aulaId: presencaAulaId, data: e.target.value });
              setPresencaItems(items);
            }
          }} />
        </fieldset>
        {presencaItems.length === 0 && <p className="text-sm text-base-content/50 py-2">Nenhum aluno vinculado a esta aula.</p>}
        {presencaItems.length > 0 && (
          <div className="mt-2">
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm font-medium">Marcar ausências:</span>
              <button type="button" className="btn btn-xs btn-ghost" onClick={() => setPresencaItems(items => items.map(i => ({ ...i, faltou: true })))}>Todos ausentes</button>
            </div>
            <div className="max-h-64 overflow-y-auto flex flex-col gap-1">
              {presencaItems.map(item => (
                <label key={item.aluno_id} className={`flex items-center gap-2 cursor-pointer p-2 rounded hover:bg-base-200 ${item.faltou ? "bg-error/10" : ""}`}>
                  <input type="checkbox" className="checkbox checkbox-sm checkbox-error" checked={item.faltou} onChange={e => setPresencaItems(items => items.map(i => i.aluno_id === item.aluno_id ? { ...i, faltou: e.target.checked } : i))} />
                  <span className="text-sm">{item.aluno_nome}</span>
                  {item.faltou && <span className="badge badge-error badge-xs ml-auto">Falta</span>}
                </label>
              ))}
            </div>
          </div>
        )}
        <div className="modal-action">
          <button className="btn" onClick={() => setPresencaAulaId(null)}>Cancelar</button>
          <button className="btn btn-primary" onClick={savePresenca} disabled={savingPresenca}>{savingPresenca ? "Salvando..." : "Salvar"}</button>
        </div>
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}

function GradeView({ aulas, materias, config, onEdit, onCellClick, onFrequencia }: { aulas: Aula[]; materias: Materia[]; config: Configuracoes | null; onEdit: (a: Aula) => void; onCellClick: (dia: string, inicio: string, fim: string) => void; onFrequencia: (aulaId: number) => void }) {
  const slots = useMemo(() => config ? getSlots(config) : [], [config]);
  const dias = useMemo(() => DIAS_SEMANA.slice(0, config?.dias_letivos_semana ?? 5), [config]);
  const todayIndex = new Date().getDay() - 1;

  if (!config || slots.length === 0) return <div className="flex justify-center p-8"><span className="loading loading-spinner" /></div>;

  return (
    <div className="overflow-auto border border-base-300 rounded-lg">
      <table className="table w-full">
        <thead>
          <tr>
            <th className="w-20 text-xs">Horário</th>
            {dias.map((dia, i) => (
              <th key={dia} className={`text-center ${i === todayIndex ? "bg-primary/10 text-primary" : ""}`}>{DIAS_ABREV[dia]}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {slots.map(slot => (
            <tr key={slot.inicio}>
              <td className="text-xs text-base-content/50 w-20 align-top pt-2">{slot.inicio}</td>
              {dias.map((dia, i) => {
                const aula = aulas.find(a => a.dia_semana === dia && a.hora_inicio === slot.inicio);
                const mat = aula ? materias.find(m => m.id === aula.materia_id) : null;
                return (
                  <td key={dia} className={`p-1 cursor-pointer hover:bg-base-200 ${i === todayIndex ? "bg-primary/5" : ""}`} onClick={() => aula ? onEdit(aula) : onCellClick(dia, slot.inicio, slot.fim)}>
                    {aula && (
                      <div className="rounded text-xs text-white p-1 leading-tight" style={{ backgroundColor: mat?.cor ?? "#6366f1" }}>
                        <div className="font-semibold truncate">{mat?.nome ?? "–"}</div>
                        <div className="opacity-80 text-[10px]">{aula.hora_inicio}–{aula.hora_fim}</div>
                        <button className="mt-0.5 opacity-70 hover:opacity-100 text-white" onClick={e => { e.stopPropagation(); onFrequencia(aula.id); }} title="Frequência"><MdEventNote size={12} /></button>
                      </div>
                    )}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ListaView({ aulas, materias, onEdit, onDeleteRequest }: { aulas: Aula[]; materias: Materia[]; onEdit: (a: Aula) => void; onDeleteRequest: (id: number) => void }) {
  return (
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
          {DIAS_SEMANA.flatMap(dia =>
            aulas
              .filter(a => a.dia_semana === dia)
              .sort((a, b) => a.hora_inicio.localeCompare(b.hora_inicio))
              .map(a => (
                <tr key={a.id}>
                  <td>{a.dia_semana}</td>
                  <td>{materias.find(m => m.id === a.materia_id)?.nome ?? "–"}</td>
                  <td>{a.hora_inicio}</td>
                  <td>{a.hora_fim}</td>
                  <td className="flex gap-2">
                    <button className="btn btn-sm btn-ghost" onClick={() => onEdit(a)}><MdEdit /></button>
                    <button className="btn btn-sm btn-ghost text-error" onClick={() => onDeleteRequest(a.id)}><MdDelete /></button>
                  </td>
                </tr>
              ))
          )}
        </tbody>
      </table>
    </div>
  );
}

