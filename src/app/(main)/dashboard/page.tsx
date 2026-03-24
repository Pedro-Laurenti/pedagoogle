"use client";
import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import {
  MdPeople,
  MdSchool,
  MdAssignment,
  MdGrade,
  MdCalendarToday,
  MdAccessTime,
  MdEventNote,
} from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import type { DashboardStats, ProximaProva, MediaMateria, Aula, Materia } from "@/types";

const DIAS_SEMANA = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];

const SHORTCUTS = [
  { href: "/alunos", label: "Alunos", icon: MdPeople, statKey: "total_alunos", color: "text-primary" },
  { href: "/materias", label: "Matérias", icon: MdSchool, statKey: "total_materias", color: "text-secondary" },
  { href: "/provas", label: "Provas", icon: MdAssignment, statKey: "total_provas", color: "text-accent" },
  { href: "/notas", label: "Notas", icon: MdGrade, statKey: "total_notas", color: "text-info" },
  { href: "/cronograma", label: "Cronograma", icon: MdCalendarToday, statKey: "total_aulas", color: "text-success" },
] as const;

export default function DashboardPage() {
  const router = useRouter();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [proximas, setProximas] = useState<ProximaProva[]>([]);
  const [medias, setMedias] = useState<MediaMateria[]>([]);
  const [aulasHoje, setAulasHoje] = useState<Aula[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);

  const load = useCallback(async () => {
    const [s, p, m, todasAulas, mats] = await Promise.all([
      invokeCmd<DashboardStats>("get_dashboard_stats"),
      invokeCmd<ProximaProva[]>("list_proximas_provas"),
      invokeCmd<MediaMateria[]>("get_medias_por_materia"),
      invokeCmd<Aula[]>("list_aulas", { semestre: null }),
      invokeCmd<Materia[]>("list_materias"),
    ]);
    setStats(s);
    setProximas(p);
    setMedias(m);
    setMaterias(mats);
    const jsDay = new Date().getDay();
    const diaHoje = jsDay === 0 ? "Domingo" : DIAS_SEMANA[jsDay - 1];
    const hoje = todasAulas
      .filter((a) => a.dia_semana === diaHoje)
      .sort((a, b) => a.hora_inicio.localeCompare(b.hora_inicio));
    setAulasHoje(hoje);
  }, []);

  useEffect(() => { load(); }, [load]);

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
        {SHORTCUTS.map(({ href, label, icon: Icon, statKey, color }) => (
          <button
            key={href}
            className="card bg-base-200 shadow hover:bg-base-300 transition-colors cursor-pointer text-left"
            onClick={() => router.push(href)}
          >
            <div className="card-body items-center text-center p-4 gap-2">
              <Icon className={`text-4xl ${color}`} />
              <p className={`text-3xl font-bold ${color}`}>
                {stats ? stats[statKey] : "–"}
              </p>
              <p className="text-sm text-base-content/70">{label}</p>
            </div>
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">
              <MdGrade className="text-info" />
              Médias por Matéria
            </h2>
            {medias.length === 0 ? (
              <p className="text-base-content/60">Nenhuma nota lançada ainda.</p>
            ) : (
              <ResponsiveContainer width="100%" height={220}>
                <BarChart data={medias} margin={{ top: 8, right: 8, left: -16, bottom: 4 }}>
                  <XAxis
                    dataKey="materia_nome"
                    tick={{ fontSize: 11 }}
                    interval={0}
                    tickFormatter={(v: string) => v.length > 10 ? v.slice(0, 10) + "…" : v}
                  />
                  <YAxis domain={[0, 10]} tick={{ fontSize: 11 }} />
                  <Tooltip
                    formatter={(value) => [typeof value === "number" ? value.toFixed(1) : value, "Média"]}
                    labelStyle={{ fontWeight: 600 }}
                  />
                  <Bar dataKey="media" radius={[4, 4, 0, 0]}>
                    {medias.map((m, i) => {
                      const mat = materias.find((mt) => mt.nome === m.materia_nome);
                      return <Cell key={i} fill={mat?.cor ?? "#6366f1"} />;
                    })}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            )}
          </div>
        </div>

        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">
              <MdEventNote className="text-accent" />
              Próximas Provas
            </h2>
            {proximas.length === 0 ? (
              <p className="text-base-content/60">Nenhuma prova agendada.</p>
            ) : (
              <ul className="space-y-2">
                {proximas.map((p) => (
                  <li key={p.id} className="flex justify-between items-center py-1 border-b border-base-300 last:border-0">
                    <div>
                      <p className="font-medium">{p.titulo}</p>
                      {p.materia_nome && (
                        <p className="text-xs text-base-content/60">{p.materia_nome}</p>
                      )}
                    </div>
                    <span className="badge badge-outline text-xs shrink-0">{p.data}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>

      <div className="card bg-base-200 shadow">
        <div className="card-body">
          <h2 className="card-title">
            <MdAccessTime className="text-success" />
            Aulas de Hoje
          </h2>
          {aulasHoje.length === 0 ? (
            <p className="text-base-content/60">Nenhuma aula hoje.</p>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3">
              {aulasHoje.map((a) => {
                const mat = materias.find((m) => m.id === a.materia_id);
                return (
                  <div
                    key={a.id}
                    className="flex items-center gap-3 p-3 bg-base-100 rounded-lg border-l-4"
                    style={{ borderColor: mat?.cor ?? "#6366f1" }}
                  >
                    <div className="min-w-0">
                      <p className="font-semibold truncate">{mat?.nome ?? "–"}</p>
                      <p className="text-sm text-base-content/60">{a.hora_inicio}–{a.hora_fim}</p>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

