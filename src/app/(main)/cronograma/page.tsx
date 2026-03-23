"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdCopyAll, MdViewModule, MdViewList } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Aula, Materia, Turma, Aluno, Configuracoes, ToastState } from "@/types";

interface AulaForm { materia_id: string; dia_semana: string; hora_inicio: string; hora_fim: string; semestre: string; bimestre: string; turma_id: string; aluno_ids: string; [k: string]: unknown; }

const DIAS = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"];
const GRADE_DIAS = ["Seg", "Ter", "Qua", "Qui", "Sex"];
const DIA_ABREV: Record<string, string> = { "Segunda": "Seg", "Terça": "Ter", "Quarta": "Qua", "Quinta": "Qui", "Sexta": "Sex" };
const MIN_START = 7 * 60;
const MIN_END = 22 * 60;
const TOTAL_MINS = MIN_END - MIN_START;
const PX_MIN = 1.5;
const SLOT = 50;
const SEMESTRE_PADRAO = "2026-1";
const EMPTY: AulaForm = { materia_id: "", dia_semana: "Segunda", hora_inicio: "08:00", hora_fim: "09:00", semestre: SEMESTRE_PADRAO, bimestre: "1", turma_id: "", aluno_ids: "[]" };

function toMins(t: string) {
  const [h, m] = t.split(":").map(Number);
  return h * 60 + m;
}

const SLOTS = Array.from({ length: Math.ceil(TOTAL_MINS / SLOT) }, (_, i) => {
  const m = MIN_START + i * SLOT;
  return `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
});

export default function CronogramaPage() {
  const [aulas, setAulas] = useState<Aula[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [form, setForm] = useState<AulaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [modoGrade, setModoGrade] = useState(true);
  const [semestreAtual, setSemestreAtual] = useState(SEMESTRE_PADRAO);
  const [showNovoSem, setShowNovoSem] = useState(false);
  const [novoSemInput, setNovoSemInput] = useState("");
  const [copiarModal, setCopiarModal] = useState(false);
  const [semestreDestino, setSemestreDestino] = useState("");
  const [copiando, setCopiando] = useState(false);

  const load = useCallback(async () => {
    const [a, m, t, al, cfg] = await Promise.all([
      invokeCmd<Aula[]>("list_aulas", { semestre: null }),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setAulas(a);
    setMaterias(m);
    setTurmas(t);
    setAlunos(al);
    setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);

  const semestres = [...new Set([...aulas.map(a => a.semestre), semestreAtual])].sort();
  const aulasFiltradas = aulas.filter(a => a.semestre === semestreAtual);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const payload = { materiaId: form.materia_id ? Number(form.materia_id) : null, diaSemana: form.dia_semana, horaInicio: form.hora_inicio, horaFim: form.hora_fim, semestre: form.semestre, bimestre: Number(form.bimestre) || 1, turmaId: form.turma_id ? Number(form.turma_id) : null, alunoIds: form.aluno_ids || "[]" };
    try {
      if (editing !== null) {
        await invokeCmd("update_aula", { id: editing, ...payload });
        notify("Aula atualizada.");
      } else {
        await invokeCmd("create_aula", payload);
        notify("Aula adicionada.");
      }
      setModal(false);
      load();
    } catch (err) {
      notify(String(err) || "Erro ao salvar.", "error");
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

  function openEdit(a: Aula) {
    setEditing(a.id);
    setForm({ materia_id: a.materia_id ? String(a.materia_id) : "", dia_semana: a.dia_semana, hora_inicio: a.hora_inicio, hora_fim: a.hora_fim, semestre: a.semestre, bimestre: String(a.bimestre ?? 1), turma_id: a.turma_id ? String(a.turma_id) : "", aluno_ids: a.aluno_ids ?? "[]" });
    setModal(true);
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
          <button className="btn btn-primary btn-sm" onClick={() => { setEditing(null); setForm({ ...EMPTY, semestre: semestreAtual }); setModal(true); }}><MdAdd size={18} /> Nova Aula</button>
        </div>
      </div>

      {modoGrade ? (
        <GradeView aulas={aulasFiltradas} materias={materias} onEdit={openEdit} />
      ) : (
        <ListaView aulas={aulasFiltradas} materias={materias} onEdit={openEdit} onDelete={handleDelete} />
      )}

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">{editing !== null ? "Editar Aula" : "Nova Aula"}</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Matéria</legend>
                <select className="select w-full" value={form.materia_id} onChange={e => setForm({ ...form, materia_id: e.target.value })}>
                  <option value="">Selecione</option>
                  {materias.map(m => <option key={m.id} value={m.id}>{m.nome}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Dia da Semana</legend>
                <select className="select w-full" value={form.dia_semana} onChange={e => setForm({ ...form, dia_semana: e.target.value })}>
                  {DIAS.map(d => <option key={d} value={d}>{d}</option>)}
                </select>
              </fieldset>
              <div className="grid grid-cols-2 gap-3">
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Início</legend>
                  <input type="time" className="input w-full" value={form.hora_inicio} onChange={e => setForm({ ...form, hora_inicio: e.target.value })} required />
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Fim</legend>
                  <input type="time" className="input w-full" value={form.hora_fim} onChange={e => setForm({ ...form, hora_fim: e.target.value })} required />
                </fieldset>
              </div>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Bimestre</legend>
                <select className="select w-full" value={form.bimestre} onChange={e => setForm({ ...form, bimestre: e.target.value })}>
                  <option value="1">1º Bimestre</option>
                  <option value="2">2º Bimestre</option>
                  <option value="3">3º Bimestre</option>
                  <option value="4">4º Bimestre</option>
                </select>
              </fieldset>
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
                  <legend className="fieldset-legend">Alunos presentes (opcional)</legend>
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
          </div>
        </div>
      )}

      {copiarModal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">Copiar Semestre</h3>
            <p className="text-sm mb-4 text-base-content/70">Copiar todas as aulas de <strong>{semestreAtual}</strong> para:</p>
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Semestre destino</legend>
              <input className="input w-full" value={semestreDestino} onChange={e => setSemestreDestino(e.target.value)} placeholder="ex: 2026-2" autoFocus />
            </fieldset>
            <div className="modal-action">
              <button className="btn" onClick={() => setCopiarModal(false)}>Cancelar</button>
              <button className="btn btn-primary" onClick={handleCopiar} disabled={copiando || !semestreDestino.trim()}>{copiando ? "Copiando..." : "Copiar"}</button>
            </div>
          </div>
        </div>
      )}

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}

function GradeView({ aulas, materias, onEdit }: { aulas: Aula[]; materias: Materia[]; onEdit: (a: Aula) => void }) {
  const [nowMins, setNowMins] = useState<number | null>(null);

  useEffect(() => {
    const update = () => {
      const now = new Date();
      setNowMins(now.getHours() * 60 + now.getMinutes());
    };
    update();
    const interval = setInterval(update, 60000);
    return () => clearInterval(interval);
  }, []);

  // JS getDay(): 0=Sun,1=Mon,...,5=Fri,6=Sat. Map to GRADE_DIAS index.
  const todayIndex = new Date().getDay() - 1; // 0=Mon..4=Fri, -1=Sun/Sat (no highlight)

  return (
    <div className="overflow-auto border border-base-300 rounded-lg" style={{ maxHeight: "70vh" }}>
      <div style={{ display: "grid", gridTemplateColumns: `55px repeat(${GRADE_DIAS.length}, 1fr)`, gridTemplateRows: `32px ${TOTAL_MINS * PX_MIN}px`, minWidth: 580 }}>
        <div className="sticky top-0 z-20 bg-base-100 border-b border-r border-base-300" />
        {GRADE_DIAS.map((abbr, i) => (
          <div key={abbr} className={`sticky top-0 z-20 border-b border-l border-base-300 flex items-center justify-center text-xs font-bold ${i === todayIndex ? "bg-primary/10 text-primary" : "bg-base-100"}`}>{abbr}</div>
        ))}
        <div className="border-r border-base-300 relative">
          {SLOTS.map(slot => (
            <div key={slot} style={{ position: "absolute", top: (toMins(slot) - MIN_START) * PX_MIN - 6, right: 4 }} className="text-xs text-base-content/50 leading-none select-none">{slot}</div>
          ))}
        </div>
        {GRADE_DIAS.map((abbr, i) => (
          <div key={abbr} className={`border-l border-base-300 relative ${i === todayIndex ? "bg-primary/5" : ""}`}>
            {SLOTS.map(slot => (
              <div key={slot} style={{ position: "absolute", top: (toMins(slot) - MIN_START) * PX_MIN, left: 0, right: 0 }} className="border-t border-base-200" />
            ))}
            {i === todayIndex && nowMins !== null && nowMins >= MIN_START && nowMins <= MIN_END && (
              <div style={{ position: "absolute", top: (nowMins - MIN_START) * PX_MIN, left: 0, right: 0, zIndex: 10 }} className="border-t-2 border-error" />
            )}
            {aulas.filter(a => DIA_ABREV[a.dia_semana] === abbr).map(a => {
              const top = (toMins(a.hora_inicio) - MIN_START) * PX_MIN;
              const height = Math.max((toMins(a.hora_fim) - toMins(a.hora_inicio)) * PX_MIN, 20);
              const mat = materias.find(m => m.id === a.materia_id);
              return (
                <div key={a.id} onClick={() => onEdit(a)} className="rounded text-xs text-white p-1 overflow-hidden cursor-pointer hover:opacity-90" style={{ position: "absolute", top, height, left: 2, right: 2, backgroundColor: mat?.cor ?? "#6366f1" }}>
                  <div className="font-semibold leading-tight truncate">{mat?.nome ?? "–"}</div>
                  <div className="leading-tight opacity-90 text-[10px]">{a.hora_inicio}–{a.hora_fim}</div>
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

function ListaView({ aulas, materias, onEdit, onDelete }: { aulas: Aula[]; materias: Materia[]; onEdit: (a: Aula) => void; onDelete: (id: number) => void }) {
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
          {DIAS.flatMap(dia =>
            aulas
              .filter(a => a.dia_semana === dia)
              .sort((a, b) => a.hora_inicio.localeCompare(b.hora_inicio))
              .map(a => (
                <tr key={a.id}>
                  <td>{a.dia_semana}</td>
                  <td>{materias.find(m => m.id === a.materia_id)?.nome ?? "-"}</td>
                  <td>{a.hora_inicio}</td>
                  <td>{a.hora_fim}</td>
                  <td className="flex gap-2">
                    <button className="btn btn-sm btn-ghost" onClick={() => onEdit(a)}><MdEdit /></button>
                    <button className="btn btn-sm btn-ghost text-error" onClick={() => onDelete(a.id)}><MdDelete /></button>
                  </td>
                </tr>
              ))
          )}
        </tbody>
      </table>
    </div>
  );
}

