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
pub fn list_provas() -> Result<Vec<Prova>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, titulo, descricao, materia_id, data, rodape, margens, valor_total,
                COALESCE(escola_override,''), COALESCE(cidade_override,''), turma_id,
                COALESCE(is_recuperacao,0), COALESCE(qr_gabarito,0),
                COALESCE(duas_colunas,0), COALESCE(paisagem,0) FROM provas ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Prova {
        id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
        materia_id: r.get(3)?, data: r.get(4)?, rodape: r.get(5)?,
        margens: r.get(6)?, valor_total: r.get(7)?,
        escola_override: r.get(8)?, cidade_override: r.get(9)?,
        turma_id: r.get(10)?,
        is_recuperacao: r.get::<_, i64>(11)? != 0,
        qr_gabarito: r.get::<_, i64>(12)? != 0,
        duas_colunas: r.get::<_, i64>(13)? != 0,
        paisagem: r.get::<_, i64>(14)? != 0,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prova(id: i64) -> Result<Prova, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, titulo, descricao, materia_id, data, rodape, margens, valor_total,
                COALESCE(escola_override,''), COALESCE(cidade_override,''), turma_id,
                COALESCE(is_recuperacao,0), COALESCE(qr_gabarito,0),
                COALESCE(duas_colunas,0), COALESCE(paisagem,0) FROM provas WHERE id=?1",
        params![id],
        |r| Ok(Prova {
            id: r.get(0)?, titulo: r.get(1)?, descricao: r.get(2)?,
            materia_id: r.get(3)?, data: r.get(4)?, rodape: r.get(5)?,
            margens: r.get(6)?, valor_total: r.get(7)?,
            escola_override: r.get(8)?, cidade_override: r.get(9)?,
            turma_id: r.get(10)?,
            is_recuperacao: r.get::<_, i64>(11)? != 0,
            qr_gabarito: r.get::<_, i64>(12)? != 0,
            duas_colunas: r.get::<_, i64>(13)? != 0,
            paisagem: r.get::<_, i64>(14)? != 0,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_prova(titulo: String, descricao: String, materia_id: Option<i64>, data: String, rodape: String, margens: String, valor_total: f64, escola_override: String, cidade_override: String, turma_id: Option<i64>, is_recuperacao: bool, qr_gabarito: bool, duas_colunas: bool, paisagem: bool) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO provas (titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao, qr_gabarito, duas_colunas, paisagem) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao as i64, qr_gabarito as i64, duas_colunas as i64, paisagem as i64],
    ).map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_prova(id: i64, titulo: String, descricao: String, materia_id: Option<i64>, data: String, rodape: String, margens: String, valor_total: f64, escola_override: String, cidade_override: String, turma_id: Option<i64>, is_recuperacao: bool, qr_gabarito: bool, duas_colunas: bool, paisagem: bool) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE provas SET titulo=?1, descricao=?2, materia_id=?3, data=?4, rodape=?5, margens=?6, valor_total=?7, escola_override=?8, cidade_override=?9, turma_id=?10, is_recuperacao=?11, qr_gabarito=?12, duas_colunas=?13, paisagem=?14 WHERE id=?15",
        params![titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao as i64, qr_gabarito as i64, duas_colunas as i64, paisagem as i64, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_prova(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM provas WHERE id=?1", params![id]).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn list_questoes(prova_id: i64) -> Result<Vec<Questao>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, COALESCE(tags,''), COALESCE(dificuldade,'médio') FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![prova_id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
            tags: r.get(8)?, dificuldade: r.get(9)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn duplicate_prova(id: i64) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let (titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao, qr_gabarito, duas_colunas, paisagem):
        (String, String, Option<i64>, String, String, String, f64, String, String, Option<i64>, i64, i64, i64, i64) = conn.query_row(
        "SELECT titulo, descricao, materia_id, data, rodape, margens, valor_total, COALESCE(escola_override,''), COALESCE(cidade_override,''), turma_id, COALESCE(is_recuperacao,0), COALESCE(qr_gabarito,0), COALESCE(duas_colunas,0), COALESCE(paisagem,0) FROM provas WHERE id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?)),
    ).map_err(|e| e.to_string())?;
    let novo_titulo = format!("{} (cópia)", titulo);
    conn.execute(
        "INSERT INTO provas (titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao, qr_gabarito, duas_colunas, paisagem) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![novo_titulo, descricao, materia_id, data, rodape, margens, valor_total, escola_override, cidade_override, turma_id, is_recuperacao, qr_gabarito, duas_colunas, paisagem],
    ).map_err(|e| e.to_string())?;
    let new_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare(
        "SELECT enunciado, tipo, opcoes, ordem, valor, linhas_resposta, COALESCE(tags,''), COALESCE(dificuldade,'médio') FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<(String, String, String, i64, f64, i64, String, String)> = stmt.query_map(params![id], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    for (enunciado, tipo, opcoes, ordem, valor, linhas_resposta, tags, dificuldade) in questoes {
        conn.execute(
            "INSERT INTO questoes (prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, tags, dificuldade) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![new_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, tags, dificuldade],
        ).map_err(|e| e.to_string())?;
    }
    Ok(new_id)
}

#[tauri::command]
pub fn replace_questoes(prova_id: i64, questoes: Vec<QuestaoInput>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let valor_total: f64 = conn.query_row(
        "SELECT valor_total FROM provas WHERE id=?1",
        params![prova_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    let soma: f64 = questoes.iter().map(|q| q.valor).sum();
    if (soma - valor_total).abs() > 0.01 {
        return Err("Soma dos pontos não corresponde ao valor total da prova".into());
    }
    conn.execute("DELETE FROM questoes WHERE prova_id=?1", params![prova_id]).map_err(|e| e.to_string())?;
    for (i, q) in questoes.iter().enumerate() {
        let opcoes = serde_json::to_string(&q.opcoes).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT INTO questoes (prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, tags, dificuldade) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![prova_id, q.enunciado, q.tipo, opcoes, i as i64, q.valor, q.linhas_resposta, q.tags, q.dificuldade],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_banco_questoes() -> Result<Vec<crate::models::BancoQuestao>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, tipo, enunciado, opcoes, valor, tags, dificuldade FROM banco_questoes ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(crate::models::BancoQuestao {
        id: r.get(0)?, tipo: r.get(1)?, enunciado: r.get(2)?,
        opcoes: r.get(3)?, valor: r.get(4)?, tags: r.get(5)?, dificuldade: r.get(6)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_banco_questao(tipo: String, enunciado: String, opcoes: String, valor: f64, tags: String, dificuldade: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO banco_questoes (tipo, enunciado, opcoes, valor, tags, dificuldade) VALUES (?1,?2,?3,?4,?5,?6)",
        params![tipo, enunciado, opcoes, valor, tags, dificuldade],
    ).map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_banco_questao(id: i64, tipo: String, enunciado: String, opcoes: String, valor: f64, tags: String, dificuldade: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE banco_questoes SET tipo=?1, enunciado=?2, opcoes=?3, valor=?4, tags=?5, dificuldade=?6 WHERE id=?7",
        params![tipo, enunciado, opcoes, valor, tags, dificuldade, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_banco_questao(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM banco_questoes WHERE id=?1", params![id]).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn import_from_banco(banco_id: i64, prova_id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let (tipo, enunciado, opcoes, valor, tags, dificuldade): (String, String, String, f64, String, String) = conn.query_row(
        "SELECT tipo, enunciado, opcoes, valor, tags, dificuldade FROM banco_questoes WHERE id=?1",
        params![banco_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
    ).map_err(|e| e.to_string())?;
    let ordem: i64 = conn.query_row(
        "SELECT COALESCE(MAX(ordem)+1, 0) FROM questoes WHERE prova_id=?1",
        params![prova_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO questoes (prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, tags, dificuldade) VALUES (?1,?2,?3,?4,?5,?6,3,?7,?8)",
        params![prova_id, enunciado, tipo, opcoes, ordem, valor, tags, dificuldade],
    ).map_err(map_db_err)?;
    Ok(())
}
