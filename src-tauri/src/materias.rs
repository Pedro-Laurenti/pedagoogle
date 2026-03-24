use rusqlite::params;
use crate::db::DbState;
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
pub fn list_materias(state: tauri::State<'_, DbState>) -> Result<Vec<Materia>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT m.id, m.nome, m.descricao, m.professor_id, p.nome, m.turma_id, \
         CASE WHEN COALESCE(t.ano,'') != '' THEN t.ano || ' - ' || t.turma ELSE COALESCE(t.nome,'') END, \
         m.carga_horaria_semanal, m.cor, COALESCE(m.icone,'MdBook') \
         FROM materias m \
         LEFT JOIN professores p ON p.id = m.professor_id \
         LEFT JOIN turmas t ON t.id = m.turma_id \
         ORDER BY m.nome"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Materia {
        id: r.get(0)?,
        nome: r.get(1)?,
        descricao: r.get(2)?,
        professor_id: r.get(3)?,
        professor_nome: r.get(4)?,
        turma_id: r.get(5)?,
        turma_nome: r.get(6)?,
        carga_horaria_semanal: r.get::<_, Option<i64>>(7)?.unwrap_or(0),
        cor: r.get::<_, Option<String>>(8)?.unwrap_or_else(|| "#6366f1".into()),
        icone: r.get::<_, Option<String>>(9)?.unwrap_or_else(|| "MdBook".into()),
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_materia(state: tauri::State<'_, DbState>, nome: String, descricao: String, professor_id: Option<i64>, turma_id: Option<i64>, carga_horaria_semanal: i64, cor: String, icone: String) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO materias (nome, descricao, professor_id, turma_id, carga_horaria_semanal, cor, icone) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![nome, descricao, professor_id, turma_id, carga_horaria_semanal, cor, icone],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: materia id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_materia(state: tauri::State<'_, DbState>, id: i64, nome: String, descricao: String, professor_id: Option<i64>, turma_id: Option<i64>, carga_horaria_semanal: i64, cor: String, icone: String) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE materias SET nome=?1, descricao=?2, professor_id=?3, turma_id=?4, carga_horaria_semanal=?5, cor=?6, icone=?7 WHERE id=?8",
        params![nome, descricao, professor_id, turma_id, carga_horaria_semanal, cor, icone, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_materia(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM provas WHERE materia_id = ?1",
        params![id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Existem provas vinculadas a esta matéria. Remova-as antes de excluir.".into());
    }
    conn.execute("DELETE FROM materias WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (materia)", id);
    Ok(())
}
