use rusqlite::{params, Connection};
use crate::db::DbState;
use crate::models::*;

/// Returns the effective grade for an aluno in a given materia, taking recuperação into account.
/// For regular provas and recuperação provas of the same aluno+materia, returns the max weighted average.
pub fn get_nota_efetiva(conn: &Connection, aluno_id: i64, materia_id: i64) -> f64 {
    struct NRow { valor: f64, peso: f64, recuperacao: bool }
    let mut stmt = match conn.prepare(
        "SELECT n.valor, p.valor_total, COALESCE(p.is_recuperacao, 0)
         FROM notas n
         JOIN provas p ON p.id = n.prova_id
         WHERE n.aluno_id = ?1 AND p.materia_id = ?2"
    ) {
        Ok(s) => s,
        Err(_) => return 0.0,
    };
    let rows: Vec<NRow> = stmt.query_map(params![aluno_id, materia_id], |r| {
        Ok(NRow {
            valor: r.get(0)?,
            peso: r.get(1)?,
            recuperacao: r.get::<_, i64>(2)? != 0,
        })
    }).ok()
    .map(|it| it.filter_map(|r| r.ok()).collect())
    .unwrap_or_default();

    let weighted_avg = |notas: &[&NRow]| -> Option<f64> {
        let peso_total: f64 = notas.iter().map(|n| n.peso).sum();
        if peso_total == 0.0 { return None; }
        let pond: f64 = notas.iter().map(|n| n.valor * n.peso).sum();
        Some(pond / peso_total)
    };

    let regular: Vec<&NRow> = rows.iter().filter(|r| !r.recuperacao).collect();
    let recuperacao: Vec<&NRow> = rows.iter().filter(|r| r.recuperacao).collect();
    let media_regular = weighted_avg(&regular);
    let media_rec = weighted_avg(&recuperacao);

    match (media_regular, media_rec) {
        (Some(r), Some(rec)) => r.max(rec),
        (Some(r), None) => r,
        (None, Some(rec)) => rec,
        (None, None) => 0.0,
    }
}

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
pub fn list_notas(state: tauri::State<'_, DbState>, bimestre: Option<i64>, ano: Option<String>, turma_id: Option<i64>, materia_id: Option<i64>, aluno_id: Option<i64>) -> Result<Vec<Nota>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT n.id, n.aluno_id, n.prova_id, n.valor, COALESCE(n.updated_at,''), n.categoria_id, cl.nome \
         FROM notas n \
         LEFT JOIN categoria_lancamentos cl ON cl.id = n.categoria_id \
         LEFT JOIN provas p ON p.id = n.prova_id \
         LEFT JOIN alunos a ON a.id = n.aluno_id \
         WHERE (?1 IS NULL OR n.aluno_id = ?1) \
         AND (?2 IS NULL OR p.bimestre = ?2) \
         AND (?3 IS NULL OR p.ano_letivo = ?3) \
         AND (?4 IS NULL OR p.turma_id = ?4 OR a.turma_id = ?4) \
         AND (?5 IS NULL OR p.materia_id = ?5) \
         ORDER BY n.id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![aluno_id, bimestre, ano, turma_id, materia_id], |r| Ok(Nota {
        id: r.get(0)?, aluno_id: r.get(1)?, prova_id: r.get(2)?,
        valor: r.get(3)?, updated_at: r.get(4)?,
        categoria_id: r.get(5)?, categoria_nome: r.get(6)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_categoria_lancamentos(state: tauri::State<'_, DbState>) -> Result<Vec<CategoriaLancamento>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, nome, cor, COALESCE(vincula_provas, 0) FROM categoria_lancamentos ORDER BY nome").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(CategoriaLancamento { id: r.get(0)?, nome: r.get(1)?, cor: r.get(2)?, vincula_provas: r.get::<_, i64>(3)? != 0 })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_categoria_lancamento(state: tauri::State<'_, DbState>, nome: String, cor: String, vincula_provas: bool) -> Result<i64, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO categoria_lancamentos (nome, cor, vincula_provas) VALUES (?1, ?2, ?3)", params![nome, cor, vincula_provas as i64]).map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_categoria_lancamento(state: tauri::State<'_, DbState>, id: i64, nome: String, cor: String, vincula_provas: bool) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE categoria_lancamentos SET nome=?1, cor=?2, vincula_provas=?3 WHERE id=?4", params![nome, cor, vincula_provas as i64, id]).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_categoria_lancamento(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM categoria_lancamentos WHERE id=?1", params![id]).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn create_nota(state: tauri::State<'_, DbState>, aluno_id: i64, prova_id: Option<i64>, valor: f64, categoria_id: Option<i64>) -> Result<i64, String> {
    if valor < 0.0 {
        return Err("Nota não pode ser negativa".into());
    }
    let conn = state.lock().map_err(|e| e.to_string())?;
    if let Some(pid) = prova_id {
        let valor_total: f64 = conn.query_row(
            "SELECT valor_total FROM provas WHERE id=?1",
            params![pid],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        if valor > valor_total {
            return Err("Nota não pode exceder o valor total da prova".into());
        }
    }
    conn.execute(
        "INSERT INTO notas (aluno_id, prova_id, valor, categoria_id) VALUES (?1,?2,?3,?4)",
        params![aluno_id, prova_id, valor, categoria_id],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: nota id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_nota(state: tauri::State<'_, DbState>, id: i64, aluno_id: i64, prova_id: Option<i64>, valor: f64, categoria_id: Option<i64>) -> Result<(), String> {
    if valor < 0.0 {
        return Err("Nota não pode ser negativa".into());
    }
    let conn = state.lock().map_err(|e| e.to_string())?;
    if let Some(pid) = prova_id {
        let valor_total: f64 = conn.query_row(
            "SELECT valor_total FROM provas WHERE id=?1",
            params![pid],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        if valor > valor_total {
            return Err("Nota não pode exceder o valor total da prova".into());
        }
    }
    conn.execute(
        "UPDATE notas SET aluno_id=?1, prova_id=?2, valor=?3, updated_at=datetime('now'), categoria_id=?4 WHERE id=?5",
        params![aluno_id, prova_id, valor, categoria_id, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_nota(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notas WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (nota)", id);
    Ok(())
}
