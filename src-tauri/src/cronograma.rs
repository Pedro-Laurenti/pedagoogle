use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

#[tauri::command]
pub fn list_aulas() -> Result<Vec<Aula>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, materia_id, dia_semana, hora_inicio, hora_fim FROM aulas ORDER BY dia_semana, hora_inicio"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Aula {
        id: r.get(0)?, materia_id: r.get(1)?, dia_semana: r.get(2)?,
        hora_inicio: r.get(3)?, hora_fim: r.get(4)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_aula(materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim) VALUES (?1,?2,?3,?4)",
        params![materia_id, dia_semana, hora_inicio, hora_fim],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_aula(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM aulas WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
