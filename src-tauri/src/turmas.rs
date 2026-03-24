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

fn turma_nome(ano: &str, turma: &str, legacy_nome: &str) -> String {
    if !ano.is_empty() {
        format!("{} - {}", ano, turma)
    } else {
        legacy_nome.to_string()
    }
}

#[tauri::command]
pub fn list_turmas() -> Result<Vec<Turma>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, COALESCE(nome,''), COALESCE(ano,''), COALESCE(turma,''), turno FROM turmas ORDER BY ano, turma")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let legacy: String = r.get(1)?;
            let ano: String = r.get(2)?;
            let turma: String = r.get(3)?;
            Ok(Turma {
                id: r.get(0)?,
                nome: turma_nome(&ano, &turma, &legacy),
                ano,
                turma,
                turno: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_turma(ano: String, turma: String, turno: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO turmas (ano, turma, turno) VALUES (?1, ?2, ?3)",
        params![ano, turma, turno],
    )
    .map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: turma id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_turma(id: i64, ano: String, turma: String, turno: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE turmas SET ano=?1, turma=?2, turno=?3 WHERE id=?4",
        params![ano, turma, turno, id],
    )
    .map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_turma(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM turmas WHERE id=?1", params![id])
        .map_err(map_db_err)?;
    log::info!("Excluído id={} (turma)", id);
    Ok(())
}

#[tauri::command]
pub fn set_turma_materias(turma_id: i64, materia_ids: Vec<i64>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM turma_materias WHERE turma_id=?1", params![turma_id]).map_err(|e| e.to_string())?;
    for mid in materia_ids {
        conn.execute(
            "INSERT OR IGNORE INTO turma_materias (turma_id, materia_id) VALUES (?1,?2)",
            params![turma_id, mid],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_turma_materias(turma_id: i64) -> Result<Vec<i64>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT materia_id FROM turma_materias WHERE turma_id=?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![turma_id], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
