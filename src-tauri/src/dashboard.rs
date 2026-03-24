use serde::{Deserialize, Serialize};
use crate::db::get_conn;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_provas: i64,
    pub total_alunos: i64,
    pub total_materias: i64,
    pub total_notas: i64,
    pub total_aulas: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProximaProva {
    pub id: i64,
    pub titulo: String,
    pub data: String,
    pub materia_nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaMateria {
    pub materia_nome: String,
    pub media: f64,
}

#[tauri::command]
pub fn get_dashboard_stats() -> Result<DashboardStats, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let total_provas: i64 = conn
        .query_row("SELECT COUNT(*) FROM provas", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_alunos: i64 = conn
        .query_row("SELECT COUNT(*) FROM alunos", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_materias: i64 = conn
        .query_row("SELECT COUNT(*) FROM materias", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_notas: i64 = conn
        .query_row("SELECT COUNT(*) FROM notas", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let total_aulas: i64 = conn
        .query_row("SELECT COUNT(*) FROM aulas", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(DashboardStats { total_provas, total_alunos, total_materias, total_notas, total_aulas })
}

#[tauri::command]
pub fn list_proximas_provas() -> Result<Vec<ProximaProva>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT p.id, p.titulo, p.data, COALESCE(m.nome, '') AS materia_nome
         FROM provas p
         LEFT JOIN materias m ON p.materia_id = m.id
         WHERE p.data >= date('now')
         ORDER BY p.data ASC
         LIMIT 5",
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        Ok(ProximaProva {
            id: r.get(0)?,
            titulo: r.get(1)?,
            data: r.get(2)?,
            materia_nome: r.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_alertas() -> Result<Vec<String>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut alertas: Vec<String> = Vec::new();

    let provas_sem_questoes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM provas p WHERE NOT EXISTS (SELECT 1 FROM questoes q WHERE q.prova_id = p.id)",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if provas_sem_questoes > 0 {
        alertas.push(format!("{} prova(s) sem questões cadastradas", provas_sem_questoes));
    }

    let alunos_sem_nota: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM alunos a WHERE NOT EXISTS (SELECT 1 FROM notas n WHERE n.aluno_id = a.id)",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if alunos_sem_nota > 0 {
        alertas.push(format!("{} aluno(s) sem nenhuma nota lançada", alunos_sem_nota));
    }

    Ok(alertas)
}

#[tauri::command]
pub fn get_medias_por_materia() -> Result<Vec<MediaMateria>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(m.nome, 'Sem matéria') AS materia_nome, AVG(n.valor) AS media
         FROM notas n
         LEFT JOIN provas p ON n.prova_id = p.id
         LEFT JOIN materias m ON p.materia_id = m.id
         GROUP BY m.id
         ORDER BY media DESC",
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        Ok(MediaMateria {
            materia_nome: r.get(0)?,
            media: r.get(1)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
