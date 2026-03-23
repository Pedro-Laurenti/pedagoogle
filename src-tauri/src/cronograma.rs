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
        "SELECT id, materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, COALESCE(aluno_ids,'[]') FROM aulas ORDER BY dia_semana, hora_inicio"
    ).map_err(|e| e.to_string())?;
    let all: Vec<Aula> = stmt.query_map([], |r| Ok(Aula {
        id: r.get(0)?, materia_id: r.get(1)?, dia_semana: r.get(2)?,
        hora_inicio: r.get(3)?, hora_fim: r.get(4)?, semestre: r.get(5)?,
        turma_id: r.get(6)?, aluno_ids: r.get(7)?,
    })).map_err(|e| e.to_string())?
       .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(match semestre {
        Some(s) => all.into_iter().filter(|a| a.semestre == s).collect(),
        None => all,
    })
}

#[tauri::command]
pub fn create_aula(materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String, turma_id: Option<i64>, aluno_ids: Option<String>) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, 0)?;
    let aids = aluno_ids.unwrap_or_else(|| "[]".into());
    conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aluno_ids) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aids],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: aula id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_aula(id: i64, materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String, turma_id: Option<i64>, aluno_ids: Option<String>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, id)?;
    let aids = aluno_ids.unwrap_or_else(|| "[]".into());
    conn.execute(
        "UPDATE aulas SET materia_id=?1, dia_semana=?2, hora_inicio=?3, hora_fim=?4, semestre=?5, turma_id=?6, aluno_ids=?7 WHERE id=?8",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aids, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_aula(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM aulas WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (aula)", id);
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
