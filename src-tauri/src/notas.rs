use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

#[tauri::command]
pub fn list_notas() -> Result<Vec<Nota>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, aluno_id, prova_id, descricao, valor FROM notas ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Nota {
        id: r.get(0)?, aluno_id: r.get(1)?, prova_id: r.get(2)?,
        descricao: r.get(3)?, valor: r.get(4)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_nota(aluno_id: i64, prova_id: Option<i64>, descricao: String, valor: f64) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO notas (aluno_id, prova_id, descricao, valor) VALUES (?1,?2,?3,?4)",
        params![aluno_id, prova_id, descricao, valor],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_nota(id: i64, aluno_id: i64, prova_id: Option<i64>, descricao: String, valor: f64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE notas SET aluno_id=?1, prova_id=?2, descricao=?3, valor=?4 WHERE id=?5",
        params![aluno_id, prova_id, descricao, valor, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_nota(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notas WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
