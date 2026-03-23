use rusqlite::params;
use crate::db::get_conn;
use crate::models::Professor;

#[tauri::command]
pub fn list_professores() -> Result<Vec<Professor>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, nome, email FROM professores ORDER BY nome").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Professor { id: r.get(0)?, nome: r.get(1)?, email: r.get(2)? })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_professor(nome: String, email: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO professores (nome, email) VALUES (?1, ?2)", params![nome, email]).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: professor id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_professor(id: i64, nome: String, email: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("UPDATE professores SET nome=?1, email=?2 WHERE id=?3", params![nome, email, id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_professor(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM materias WHERE professor_id = ?1",
        params![id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Existem matérias vinculadas a este professor. Remova o vínculo antes de excluir.".into());
    }
    conn.execute("DELETE FROM professores WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    log::info!("Excluído id={} (professor)", id);
    Ok(())
}
