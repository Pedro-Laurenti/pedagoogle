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

const ATIVIDADE_SELECT: &str = "SELECT a.id, a.titulo, a.descricao, a.materia_id,
        COALESCE(a.bimestre,1), COALESCE(a.ano_letivo,''), a.valor_total, a.turma_id,
        a.vale_nota, COALESCE(a.updated_at,''),
        (SELECT COUNT(*) FROM questoes_atividade q WHERE q.atividade_id = a.id) FROM atividades a";

fn map_atividade(r: &rusqlite::Row) -> rusqlite::Result<Atividade> {
    Ok(Atividade {
        id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
        materia_id: r.get(3)?, bimestre: r.get(4)?, ano_letivo: r.get(5)?,
        valor_total: r.get(6)?, turma_id: r.get(7)?,
        vale_nota: r.get::<_, i64>(8)? != 0,
        updated_at: r.get(9)?,
        questoes_count: r.get(10)?,
    })
}

#[tauri::command]
pub fn list_atividades(state: tauri::State<'_, DbState>) -> Result<Vec<Atividade>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(&format!("{} ORDER BY a.id DESC", ATIVIDADE_SELECT)).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_atividade).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_atividade(state: tauri::State<'_, DbState>, id: i64) -> Result<Atividade, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        &format!("{} WHERE a.id=?1", ATIVIDADE_SELECT),
        params![id],
        map_atividade,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_atividade(state: tauri::State<'_, DbState>, titulo: String, descricao: String, materia_id: Option<i64>, bimestre: i64, ano_letivo: String, turma_id: Option<i64>, valor_total: f64, vale_nota: bool) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO atividades (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota as i64],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: atividade id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_atividade(state: tauri::State<'_, DbState>, id: i64, titulo: String, descricao: String, materia_id: Option<i64>, bimestre: i64, ano_letivo: String, turma_id: Option<i64>, valor_total: f64, vale_nota: bool) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE atividades SET titulo=?1, descricao=?2, materia_id=?3, bimestre=?4, ano_letivo=?5, turma_id=?6, valor_total=?7, vale_nota=?8, updated_at=datetime('now') WHERE id=?9",
        params![titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota as i64, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_atividade(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM atividades WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (atividade)", id);
    Ok(())
}

#[tauri::command]
pub fn duplicate_atividade(state: tauri::State<'_, DbState>, id: i64) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota):
        (String, String, Option<i64>, i64, String, Option<i64>, f64, i64) = conn.query_row(
        "SELECT titulo, descricao, materia_id, COALESCE(bimestre,1), COALESCE(ano_letivo,''), turma_id, valor_total, vale_nota FROM atividades WHERE id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
    ).map_err(|e| e.to_string())?;
    let novo_titulo = format!("{} (cópia)", titulo);
    conn.execute(
        "INSERT INTO atividades (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![novo_titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, vale_nota],
    ).map_err(|e| e.to_string())?;
    let new_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare(
        "SELECT enunciado, tipo, opcoes, ordem, valor, linhas_resposta FROM questoes_atividade WHERE atividade_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<(String, String, String, i64, f64, i64)> = stmt.query_map(params![id], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    for (enunciado, tipo, opcoes, ordem, valor, linhas_resposta) in questoes {
        conn.execute(
            "INSERT INTO questoes_atividade (atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![new_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta],
        ).map_err(|e| e.to_string())?;
    }
    Ok(new_id)
}

#[tauri::command]
pub fn list_questoes_atividade(state: tauri::State<'_, DbState>, atividade_id: i64) -> Result<Vec<AtividadeQuestao>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta FROM questoes_atividade WHERE atividade_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![atividade_id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(AtividadeQuestao {
            id: r.get(0)?, atividade_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn replace_questoes_atividade(state: tauri::State<'_, DbState>, atividade_id: i64, questoes: Vec<QuestaoInput>) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let (valor_total, vale_nota): (f64, i64) = conn.query_row(
        "SELECT valor_total, vale_nota FROM atividades WHERE id=?1",
        params![atividade_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| e.to_string())?;
    if vale_nota != 0 {
        let questoes_reais: Vec<&QuestaoInput> = questoes.iter().filter(|q| q.tipo != "texto").collect();
        let soma: f64 = questoes_reais.iter().map(|q| q.valor).sum();
        if !questoes_reais.is_empty() && (soma - valor_total).abs() > 0.01 {
            return Err("Soma dos pontos não corresponde ao valor total da atividade".into());
        }
    }
    conn.execute("DELETE FROM questoes_atividade WHERE atividade_id=?1", params![atividade_id]).map_err(|e| e.to_string())?;
    for (i, q) in questoes.iter().enumerate() {
        let opcoes = serde_json::to_string(&q.opcoes).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT INTO questoes_atividade (atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![atividade_id, q.enunciado, q.tipo, opcoes, i as i64, q.valor, q.linhas_resposta],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}
