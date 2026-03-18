use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

#[tauri::command]
pub fn list_materias() -> Result<Vec<Materia>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, nome, descricao, professor FROM materias ORDER BY nome").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Materia { id: r.get(0)?, nome: r.get(1)?, descricao: r.get(2)?, professor: r.get(3)? })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_materia(nome: String, descricao: String, professor: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO materias (nome, descricao, professor) VALUES (?1, ?2, ?3)", params![nome, descricao, professor]).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_materia(id: i64, nome: String, descricao: String, professor: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("UPDATE materias SET nome=?1, descricao=?2, professor=?3 WHERE id=?4", params![nome, descricao, professor, id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_materia(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM materias WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
