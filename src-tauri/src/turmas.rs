use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

fn map_db_err(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("UNIQUE constraint failed") {
        return "Já existe um registro com esse valor.".into();
    }
    if s.contains("FOREIGN KEY constraint failed") {
        return "Não é possível excluir: existem registros vinculados.".into();
    }
    s
}

#[tauri::command]
pub fn list_turmas() -> Result<Vec<Turma>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, nome, ano_letivo, turno FROM turmas ORDER BY nome")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Turma {
                id: r.get(0)?,
                nome: r.get(1)?,
                ano_letivo: r.get(2)?,
                turno: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_turma(nome: String, ano_letivo: String, turno: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO turmas (nome, ano_letivo, turno) VALUES (?1, ?2, ?3)",
        params![nome, ano_letivo, turno],
    )
    .map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_turma(id: i64, nome: String, ano_letivo: String, turno: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE turmas SET nome=?1, ano_letivo=?2, turno=?3 WHERE id=?4",
        params![nome, ano_letivo, turno, id],
    )
    .map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_turma(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM turmas WHERE id=?1", params![id])
        .map_err(map_db_err)?;
    Ok(())
}
