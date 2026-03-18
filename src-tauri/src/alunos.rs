use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

#[tauri::command]
pub fn list_alunos() -> Result<Vec<Aluno>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, nome FROM alunos ORDER BY nome").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Aluno { id: r.get(0)?, nome: r.get(1)? })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_aluno(nome: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO alunos (nome) VALUES (?1)", params![nome]).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_aluno(id: i64, nome: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("UPDATE alunos SET nome=?1 WHERE id=?2", params![nome, id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_aluno(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM alunos WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
