"use client";
import { useState, useEffect, useCallback } from "react";
import { invokeCmd } from "@/utils/tauri";
import type { DashboardStats, ProximaProva, MediaMateria } from "@/types";

export default function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [proximas, setProximas] = useState<ProximaProva[]>([]);
  const [alertas, setAlertas] = useState<string[]>([]);
  const [medias, setMedias] = useState<MediaMateria[]>([]);

  const load = useCallback(async () => {
    const [s, p, a, m] = await Promise.all([
      invokeCmd<DashboardStats>("get_dashboard_stats"),
      invokeCmd<ProximaProva[]>("list_proximas_provas"),
      invokeCmd<string[]>("get_alertas"),
      invokeCmd<MediaMateria[]>("get_medias_por_materia"),
    ]);
    setStats(s);
    setProximas(p);
    setAlertas(a);
    setMedias(m);
  }, []);

  useEffect(() => { load(); }, [load]);

  const maxMedia = medias.length > 0 ? Math.max(...medias.map((m) => m.media)) : 10;

  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Dashboard</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Provas</h2>
            <p className="text-4xl font-bold text-primary">
              {stats ? stats.total_provas : "-"}
            </p>
          </div>
        </div>
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Alunos</h2>
            <p className="text-4xl font-bold text-primary">
              {stats ? stats.total_alunos : "-"}
            </p>
          </div>
        </div>
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Matérias</h2>
            <p className="text-4xl font-bold text-primary">
              {stats ? stats.total_materias : "-"}
            </p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Próximas Provas</h2>
            {proximas.length === 0 ? (
              <p className="text-base-content/60">Nenhuma prova agendada</p>
            ) : (
              <ul className="space-y-2">
                {proximas.map((p) => (
                  <li key={p.id} className="flex justify-between items-center">
                    <span className="font-medium">{p.titulo}</span>
                    <span className="text-sm text-base-content/60">
                      {p.data}{p.materia_nome ? ` — ${p.materia_nome}` : ""}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Alertas</h2>
            {alertas.length === 0 ? (
              <p className="text-base-content/60">Nenhum alerta</p>
            ) : (
              <ul className="list-disc list-inside space-y-1">
                {alertas.map((a, i) => (
                  <li key={i} className="text-warning">{a}</li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>

      {medias.length > 0 && (
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title mb-4">Médias por Matéria</h2>
            <div className="space-y-3">
              {medias.map((m) => (
                <div key={m.materia_nome}>
                  <div className="flex justify-between text-sm mb-1">
                    <span>{m.materia_nome}</span>
                    <span>{m.media.toFixed(1)}</span>
                  </div>
                  <div className="w-full bg-base-300 rounded-full h-3">
                    <div
                      className="bg-primary h-3 rounded-full"
                      style={{ width: `${(m.media / maxMedia) * 100}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

