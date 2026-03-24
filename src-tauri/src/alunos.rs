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
pub fn list_alunos(state: tauri::State<'_, DbState>) -> Result<Vec<Aluno>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.nome, a.turma_id, \
             CASE WHEN COALESCE(t.ano,'') != '' THEN t.ano || ' - ' || t.turma ELSE COALESCE(t.nome,'') END, \
             COALESCE(a.foto_path,''), COALESCE(a.updated_at,'') \
             FROM alunos a LEFT JOIN turmas t ON a.turma_id = t.id \
             ORDER BY a.nome",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Aluno {
                id: r.get(0)?,
                nome: r.get(1)?,
                turma_id: r.get(2)?,
                turma_nome: r.get(3)?,
                foto_path: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_alunos_page(state: tauri::State<'_, DbState>, page: i64, per_page: i64) -> Result<Vec<Aluno>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.nome, a.turma_id, \
             CASE WHEN COALESCE(t.ano,'') != '' THEN t.ano || ' - ' || t.turma ELSE COALESCE(t.nome,'') END, \
             COALESCE(a.foto_path,''), COALESCE(a.updated_at,'') \
             FROM alunos a LEFT JOIN turmas t ON a.turma_id = t.id \
             ORDER BY a.nome LIMIT ?1 OFFSET (?2-1)*?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![per_page, page], |r| {
            Ok(Aluno {
                id: r.get(0)?,
                nome: r.get(1)?,
                turma_id: r.get(2)?,
                turma_nome: r.get(3)?,
                foto_path: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_aluno(state: tauri::State<'_, DbState>, nome: String, turma_id: Option<i64>, foto_path: String) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO alunos (nome, turma_id, foto_path) VALUES (?1, ?2, ?3)",
        params![nome, turma_id, foto_path],
    )
    .map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: aluno id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_aluno(state: tauri::State<'_, DbState>, id: i64, nome: String, turma_id: Option<i64>, foto_path: String) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE alunos SET nome=?1, turma_id=?2, foto_path=?3, updated_at=datetime('now') WHERE id=?4",
        params![nome, turma_id, foto_path, id],
    )
    .map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn set_aluno_materias(state: tauri::State<'_, DbState>, aluno_id: i64, materia_ids: Vec<i64>) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM aluno_materias WHERE aluno_id=?1", params![aluno_id]).map_err(|e| e.to_string())?;
    for mid in materia_ids {
        conn.execute(
            "INSERT OR IGNORE INTO aluno_materias (aluno_id, materia_id) VALUES (?1,?2)",
            params![aluno_id, mid],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_aluno_materias(state: tauri::State<'_, DbState>, aluno_id: i64) -> Result<Vec<i64>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT materia_id FROM aluno_materias WHERE aluno_id=?1").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![aluno_id], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_aluno(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notas WHERE aluno_id = ?1",
        params![id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Existem notas vinculadas a este aluno. Remova-as antes de excluir.".into());
    }
    conn.execute("DELETE FROM alunos WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (aluno)", id);
    Ok(())
}
