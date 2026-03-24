use rusqlite::params;
use crate::db::get_conn;
use crate::models::{Professor, ProfessorCronograma, ProfessorCronogramaInput};

#[tauri::command]
pub fn list_professores() -> Result<Vec<Professor>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, nome, COALESCE(aulas_por_semana,0) FROM professores ORDER BY nome"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Professor {
        id: r.get(0)?, nome: r.get(1)?, aulas_por_semana: r.get(2)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_professor(nome: String, aulas_por_semana: i64) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO professores (nome, aulas_por_semana) VALUES (?1,?2)",
        params![nome, aulas_por_semana],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: professor id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_professor(id: i64, nome: String, aulas_por_semana: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE professores SET nome=?1, aulas_por_semana=?2 WHERE id=?3",
        params![nome, aulas_por_semana, id],
    ).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn set_professor_materias(professor_id: i64, materia_ids: Vec<i64>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM professor_materias WHERE professor_id=?1", params![professor_id]).map_err(|e| e.to_string())?;
    for mid in materia_ids {
        conn.execute(
            "INSERT OR IGNORE INTO professor_materias (professor_id, materia_id) VALUES (?1,?2)",
            params![professor_id, mid],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_professor_turmas(professor_id: i64, turma_ids: Vec<i64>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM professor_turmas WHERE professor_id=?1", params![professor_id]).map_err(|e| e.to_string())?;
    for tid in turma_ids {
        conn.execute(
            "INSERT OR IGNORE INTO professor_turmas (professor_id, turma_id) VALUES (?1,?2)",
            params![professor_id, tid],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_professor_materias(professor_id: i64) -> Result<Vec<i64>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT materia_id FROM professor_materias WHERE professor_id=?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![professor_id], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_professor_turmas(professor_id: i64) -> Result<Vec<i64>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT turma_id FROM professor_turmas WHERE professor_id=?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![professor_id], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_professor_cronograma(professor_id: i64) -> Result<Vec<ProfessorCronograma>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, professor_id, titulo, dia_semana, hora_inicio, hora_fim, cor, recorrente \
         FROM professor_cronograma WHERE professor_id=?1 ORDER BY dia_semana, hora_inicio"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![professor_id], |r| Ok(ProfessorCronograma {
        id: r.get(0)?, professor_id: r.get(1)?, titulo: r.get(2)?,
        dia_semana: r.get(3)?, hora_inicio: r.get(4)?, hora_fim: r.get(5)?,
        cor: r.get(6)?, recorrente: r.get::<_, i64>(7)? != 0,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_professor_cronograma(professor_id: i64, eventos: Vec<ProfessorCronogramaInput>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM professor_cronograma WHERE professor_id=?1", params![professor_id]).map_err(|e| e.to_string())?;
    for ev in eventos {
        conn.execute(
            "INSERT INTO professor_cronograma (professor_id, titulo, dia_semana, hora_inicio, hora_fim, cor, recorrente) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![professor_id, ev.titulo, ev.dia_semana, ev.hora_inicio, ev.hora_fim, ev.cor, ev.recorrente as i64],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}
