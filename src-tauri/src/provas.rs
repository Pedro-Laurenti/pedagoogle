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

const PROVA_SELECT: &str = "SELECT p.id, p.titulo, p.descricao, p.materia_id,
        COALESCE(p.bimestre,1), COALESCE(p.ano_letivo,''), p.valor_total, p.turma_id,
        COALESCE(p.updated_at,''),
        (SELECT COUNT(*) FROM questoes q WHERE q.prova_id = p.id) FROM provas p";

fn map_prova(r: &rusqlite::Row) -> rusqlite::Result<Prova> {
    Ok(Prova {
        id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
        materia_id: r.get(3)?, bimestre: r.get(4)?, ano_letivo: r.get(5)?,
        valor_total: r.get(6)?, turma_id: r.get(7)?, updated_at: r.get(8)?,
        questoes_count: r.get(9)?,
    })
}

fn gerar_titulo(conn: &rusqlite::Connection, materia_id: Option<i64>, bimestre: i64) -> String {
    let nome: String = materia_id
        .and_then(|mid| conn.query_row("SELECT nome FROM materias WHERE id=?1", params![mid], |r| r.get(0)).ok())
        .unwrap_or_else(|| "MATÉRIA".to_string());
    format!("AVALIAÇÃO DE {} - {}º BIMESTRE", nome.to_uppercase(), bimestre)
}

#[tauri::command]
pub fn list_provas(state: tauri::State<'_, DbState>) -> Result<Vec<Prova>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(&format!("{} ORDER BY p.id DESC", PROVA_SELECT)).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_prova).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_provas_page(state: tauri::State<'_, DbState>, page: i64, per_page: i64) -> Result<Vec<Prova>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(&format!("{} ORDER BY p.id DESC LIMIT ?1 OFFSET (?2-1)*?1", PROVA_SELECT)).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![per_page, page], map_prova).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prova(state: tauri::State<'_, DbState>, id: i64) -> Result<Prova, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        &format!("{} WHERE p.id=?1", PROVA_SELECT),
        params![id],
        map_prova,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_prova(state: tauri::State<'_, DbState>, descricao: String, materia_id: Option<i64>, bimestre: i64, ano_letivo: String, turma_id: Option<i64>, valor_total: f64) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let titulo = gerar_titulo(&conn, materia_id, bimestre);
    conn.execute(
        "INSERT INTO provas (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: prova id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_prova(state: tauri::State<'_, DbState>, id: i64, descricao: String, materia_id: Option<i64>, bimestre: i64, ano_letivo: String, turma_id: Option<i64>, valor_total: f64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let titulo = gerar_titulo(&conn, materia_id, bimestre);
    conn.execute(
        "UPDATE provas SET titulo=?1, descricao=?2, materia_id=?3, bimestre=?4, ano_letivo=?5, turma_id=?6, valor_total=?7, updated_at=datetime('now') WHERE id=?8",
        params![titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_prova(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM provas WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (prova)", id);
    Ok(())
}

#[tauri::command]
pub fn duplicate_prova(state: tauri::State<'_, DbState>, id: i64) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total):
        (String, String, Option<i64>, i64, String, Option<i64>, f64) = conn.query_row(
        "SELECT titulo, descricao, materia_id, COALESCE(bimestre,1), COALESCE(ano_letivo,''), turma_id, valor_total FROM provas WHERE id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
    ).map_err(|e| e.to_string())?;
    let novo_titulo = format!("{} (cópia)", titulo);
    conn.execute(
        "INSERT INTO provas (titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![novo_titulo, descricao, materia_id, bimestre, ano_letivo, turma_id, valor_total],
    ).map_err(|e| e.to_string())?;
    let new_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare(
        "SELECT enunciado, tipo, opcoes, ordem, valor, linhas_resposta FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<(String, String, String, i64, f64, i64)> = stmt.query_map(params![id], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    for (enunciado, tipo, opcoes, ordem, valor, linhas_resposta) in questoes {
        conn.execute(
            "INSERT INTO questoes (prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![new_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta],
        ).map_err(|e| e.to_string())?;
    }
    Ok(new_id)
}

#[tauri::command]
pub fn list_questoes(state: tauri::State<'_, DbState>, prova_id: i64) -> Result<Vec<Questao>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![prova_id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn replace_questoes(state: tauri::State<'_, DbState>, prova_id: i64, questoes: Vec<QuestaoInput>) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let valor_total: f64 = conn.query_row(
        "SELECT valor_total FROM provas WHERE id=?1",
        params![prova_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    let questoes_reais: Vec<&QuestaoInput> = questoes.iter().filter(|q| q.tipo != "texto").collect();
    let soma: f64 = questoes_reais.iter().map(|q| q.valor).sum();
    if !questoes_reais.is_empty() && (soma - valor_total).abs() > 0.01 {
        return Err("Soma dos pontos não corresponde ao valor total da prova".into());
    }
    conn.execute("DELETE FROM questoes WHERE prova_id=?1", params![prova_id]).map_err(|e| e.to_string())?;
    for (i, q) in questoes.iter().enumerate() {
        let opcoes = serde_json::to_string(&q.opcoes).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT INTO questoes (prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![prova_id, q.enunciado, q.tipo, opcoes, i as i64, q.valor, q.linhas_resposta],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}
