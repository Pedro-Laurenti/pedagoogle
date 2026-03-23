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

fn check_conflito(conn: &rusqlite::Connection, dia_semana: &str, hora_inicio: &str, hora_fim: &str, semestre: &str, excluir_id: i64) -> Result<(), String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM aulas WHERE dia_semana=?1 AND id!=?2 AND hora_inicio<?3 AND hora_fim>?4 AND semestre=?5",
        params![dia_semana, excluir_id, hora_fim, hora_inicio, semestre],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    if count > 0 { return Err("Conflito de horário com outra aula".into()); }
    Ok(())
}

#[tauri::command]
pub fn list_aulas(semestre: Option<String>) -> Result<Vec<Aula>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, materia_id, dia_semana, hora_inicio, hora_fim, semestre FROM aulas ORDER BY dia_semana, hora_inicio"
    ).map_err(|e| e.to_string())?;
    let all: Vec<Aula> = stmt.query_map([], |r| Ok(Aula {
        id: r.get(0)?, materia_id: r.get(1)?, dia_semana: r.get(2)?,
        hora_inicio: r.get(3)?, hora_fim: r.get(4)?, semestre: r.get(5)?,
    })).map_err(|e| e.to_string())?
       .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(match semestre {
        Some(s) => all.into_iter().filter(|a| a.semestre == s).collect(),
        None => all,
    })
}

#[tauri::command]
pub fn create_aula(materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, 0)?;
    conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre) VALUES (?1,?2,?3,?4,?5)",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre],
    ).map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_aula(id: i64, materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, id)?;
    conn.execute(
        "UPDATE aulas SET materia_id=?1, dia_semana=?2, hora_inicio=?3, hora_fim=?4, semestre=?5 WHERE id=?6",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_aula(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM aulas WHERE id=?1", params![id]).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn copy_semestre(de: String, para: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let count = conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre) SELECT materia_id, dia_semana, hora_inicio, hora_fim, ?1 FROM aulas WHERE semestre=?2",
        params![para, de],
    ).map_err(|e| e.to_string())?;
    Ok(count as i64)
}
