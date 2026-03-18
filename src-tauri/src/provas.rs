use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;

#[tauri::command]
pub fn list_provas() -> Result<Vec<Prova>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, titulo, descricao, materia_id, data, rodape, margens, valor_total FROM provas ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Prova {
        id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
        materia_id: r.get(3)?, data: r.get(4)?, rodape: r.get(5)?,
        margens: r.get(6)?, valor_total: r.get(7)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prova(id: i64) -> Result<Prova, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, titulo, descricao, materia_id, data, rodape, margens, valor_total FROM provas WHERE id=?1",
        params![id],
        |r| Ok(Prova {
            id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
            materia_id: r.get(3)?, data: r.get(4)?, rodape: r.get(5)?,
            margens: r.get(6)?, valor_total: r.get(7)?,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_prova(titulo: String, descricao: String, materia_id: Option<i64>, data: String, rodape: String, margens: String, valor_total: f64) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO provas (titulo, descricao, materia_id, data, rodape, margens, valor_total) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![titulo, descricao, materia_id, data, rodape, margens, valor_total],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_prova(id: i64, titulo: String, descricao: String, materia_id: Option<i64>, data: String, rodape: String, margens: String, valor_total: f64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE provas SET titulo=?1, descricao=?2, materia_id=?3, data=?4, rodape=?5, margens=?6, valor_total=?7 WHERE id=?8",
        params![titulo, descricao, materia_id, data, rodape, margens, valor_total, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_prova(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM provas WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_questoes(prova_id: i64) -> Result<Vec<Questao>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
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
pub fn replace_questoes(prova_id: i64, questoes: Vec<QuestaoInput>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
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
