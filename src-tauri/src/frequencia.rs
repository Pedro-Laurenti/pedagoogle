use rusqlite::params;
use crate::db::get_conn;
use crate::models::{Presenca, FrequenciaMateria};

#[tauri::command]
pub fn list_presencas(aula_id: i64) -> Result<Vec<Presenca>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT p.id, p.aluno_id, p.aula_id, a.nome, p.data, p.presente
         FROM presencas p
         JOIN alunos a ON a.id = p.aluno_id
         WHERE p.aula_id = ?1
         ORDER BY a.nome"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![aula_id], |r| Ok(Presenca {
        id: r.get(0)?,
        aluno_id: r.get(1)?,
        aula_id: r.get(2)?,
        aluno_nome: r.get(3)?,
        data: r.get(4)?,
        presente: r.get::<_, i64>(5)? != 0,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_presenca(aluno_id: i64, aula_id: i64, data: String, presente: bool) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO presencas (aluno_id, aula_id, data, presente) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(aluno_id, aula_id) DO UPDATE SET data=excluded.data, presente=excluded.presente",
        params![aluno_id, aula_id, data, presente as i64],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_frequencia_aluno(aluno_id: i64) -> Result<Vec<FrequenciaMateria>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT m.nome,
                COUNT(*) as total_aulas,
                SUM(CASE WHEN p.presente = 1 THEN 1 ELSE 0 END) as presencas
         FROM presencas p
         JOIN aulas a ON a.id = p.aula_id
         JOIN materias m ON m.id = a.materia_id
         WHERE p.aluno_id = ?1
         GROUP BY m.id, m.nome
         ORDER BY m.nome"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![aluno_id], |r| {
        let total_aulas: i64 = r.get(1)?;
        let presencas: i64 = r.get(2)?;
        let percentual = if total_aulas > 0 {
            presencas as f64 / total_aulas as f64 * 100.0
        } else {
            0.0
        };
        Ok(FrequenciaMateria {
            materia_nome: r.get(0)?,
            total_aulas,
            presencas,
            percentual,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
