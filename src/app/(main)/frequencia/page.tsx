"use client";
import { useState, useEffect, useCallback } from "react";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Aula, Aluno, Materia, Presenca, FrequenciaMateria, ToastState } from "@/types";

export default function FrequenciaPage() {
  const [aulas, setAulas] = useState<Aula[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [selectedAulaId, setSelectedAulaId] = useState<string>("");
  const [data, setData] = useState<string>(new Date().toISOString().slice(0, 10));
  const [presencas, setPresencas] = useState<Record<number, boolean>>({});
  const [turmaAlunos, setTurmaAlunos] = useState<Aluno[]>([]);
  const [selectedAlunoId, setSelectedAlunoId] = useState<string>("");
  const [frequencia, setFrequencia] = useState<FrequenciaMateria[]>([]);
  const [toast, setToast] = useState<ToastState | null>(null);

  const notify = (message: string, type: ToastState["type"] = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  const load = useCallback(async () => {
    const [a, m, al] = await Promise.all([
      invokeCmd<Aula[]>("list_aulas"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Aluno[]>("list_alunos"),
    ]);
    setAulas(a);
    setMaterias(m);
    setAlunos(al);
  }, []);

  useEffect(() => { load(); }, [load]);

  useEffect(() => {
    if (!selectedAulaId) {
      setTurmaAlunos([]);
      setPresencas({});
      return;
    }
    const aula = aulas.find((a) => a.id === Number(selectedAulaId));
    if (!aula) return;

    const materia = materias.find((m) => m.id === aula.materia_id);
    const turmaId = materia?.turma_id ?? null;
    const filtered = turmaId ? alunos.filter((al) => al.turma_id === turmaId) : alunos;
    setTurmaAlunos(filtered);

    invokeCmd<Presenca[]>("list_presencas", { aulaId: Number(selectedAulaId) }).then((ps) => {
      const map: Record<number, boolean> = {};
      // Default all to true for new sessions
      filtered.forEach((al) => { map[al.id] = true; });
      // Override with existing records
      ps.forEach((p) => { map[p.aluno_id] = p.presente; });
      setPresencas(map);
    });
  }, [selectedAulaId, aulas, materias, alunos]);

  async function handlePresencaChange(alunoId: number, presente: boolean) {
    if (!selectedAulaId || !data) {
      notify("Selecione uma aula e uma data.", "warning");
      return;
    }
    setPresencas((prev) => ({ ...prev, [alunoId]: presente }));
    try {
      await invokeCmd("upsert_presenca", {
        alunoId,
        aulaId: Number(selectedAulaId),
        data,
        presente,
      });
    } catch (e) {
      notify(`Erro ao salvar presença: ${e}`, "error");
    }
  }

  async function loadFrequencia(alunoId: string) {
    setSelectedAlunoId(alunoId);
    if (!alunoId) { setFrequencia([]); return; }
    try {
      const freq = await invokeCmd<FrequenciaMateria[]>("get_frequencia_aluno", { alunoId: Number(alunoId) });
      setFrequencia(freq);
    } catch (e) {
      notify(`Erro ao carregar frequência: ${e}`, "error");
    }
  }

  const aulaLabel = (a: Aula) => {
    const mat = materias.find((m) => m.id === a.materia_id);
    return `${a.dia_semana} ${a.hora_inicio}–${a.hora_fim}${mat ? ` — ${mat.nome}` : ""}`;
  };

  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Frequência</h1>

      {/* Aula selector + data */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Aula</legend>
          <select
            className="select w-full"
            value={selectedAulaId}
            onChange={(e) => setSelectedAulaId(e.target.value)}
          >
            <option value="">Selecione uma aula</option>
            {aulas.map((a) => (
              <option key={a.id} value={a.id}>{aulaLabel(a)}</option>
            ))}
          </select>
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Data</legend>
          <input
            type="date"
            className="input w-full"
            value={data}
            onChange={(e) => setData(e.target.value)}
          />
        </fieldset>
      </div>

      {/* Attendance grid */}
      {selectedAulaId && (
        <div className="mb-8">
          <h2 className="text-xl font-semibold mb-3">Lista de Presença</h2>
          {turmaAlunos.length === 0 ? (
            <p className="text-base-content/50 text-sm">Nenhum aluno encontrado para esta aula.</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="table table-zebra w-full">
                <thead>
                  <tr>
                    <th>Aluno</th>
                    <th className="text-center">Presente</th>
                  </tr>
                </thead>
                <tbody>
                  {turmaAlunos.map((al) => (
                    <tr key={al.id}>
                      <td>{al.nome}</td>
                      <td className="text-center">
                        <input
                          type="checkbox"
                          className="checkbox"
                          checked={presencas[al.id] ?? true}
                          onChange={(e) => handlePresencaChange(al.id, e.target.checked)}
                        />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* Frequency report */}
      <div>
        <h2 className="text-xl font-semibold mb-3">Relatório de Frequência</h2>
        <fieldset className="fieldset mb-4 max-w-sm">
          <legend className="fieldset-legend">Aluno</legend>
          <select
            className="select w-full"
            value={selectedAlunoId}
            onChange={(e) => loadFrequencia(e.target.value)}
          >
            <option value="">Selecione um aluno</option>
            {alunos.map((a) => (
              <option key={a.id} value={a.id}>{a.nome}</option>
            ))}
          </select>
        </fieldset>

        {selectedAlunoId && frequencia.length === 0 && (
          <p className="text-base-content/50 text-sm">Nenhum registro de presença encontrado.</p>
        )}

        {frequencia.length > 0 && (
          <div className="overflow-x-auto">
            <table className="table table-zebra w-full">
              <thead>
                <tr>
                  <th>Matéria</th>
                  <th className="text-right">Total de Aulas</th>
                  <th className="text-right">Presenças</th>
                  <th className="text-right">Frequência</th>
                </tr>
              </thead>
              <tbody>
                {frequencia.map((f) => (
                  <tr key={f.materia_nome}>
                    <td>{f.materia_nome}</td>
                    <td className="text-right">{f.total_aulas}</td>
                    <td className="text-right">{f.presencas}</td>
                    <td className="text-right">
                      <span className={`font-semibold ${f.percentual >= 75 ? "text-success" : "text-error"}`}>
                        {f.percentual.toFixed(1)}%
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
