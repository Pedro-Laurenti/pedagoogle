use rusqlite::{params, Connection};
use crate::db::get_conn;
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
pub fn list_notas() -> Result<Vec<Nota>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, aluno_id, prova_id, descricao, valor, COALESCE(updated_at,'') FROM notas ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(Nota {
        id: r.get(0)?, aluno_id: r.get(1)?, prova_id: r.get(2)?,
        descricao: r.get(3)?, valor: r.get(4)?, updated_at: r.get(5)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_nota(aluno_id: i64, prova_id: Option<i64>, descricao: String, valor: f64) -> Result<i64, String> {
    if valor < 0.0 {
        return Err("Nota não pode ser negativa".into());
    }
    let conn = get_conn().map_err(|e| e.to_string())?;
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
        "INSERT INTO notas (aluno_id, prova_id, descricao, valor) VALUES (?1,?2,?3,?4)",
        params![aluno_id, prova_id, descricao, valor],
    ).map_err(map_db_err)?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_nota(id: i64, aluno_id: i64, prova_id: Option<i64>, descricao: String, valor: f64) -> Result<(), String> {
    if valor < 0.0 {
        return Err("Nota não pode ser negativa".into());
    }
    let conn = get_conn().map_err(|e| e.to_string())?;
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
        "UPDATE notas SET aluno_id=?1, prova_id=?2, descricao=?3, valor=?4, updated_at=datetime('now') WHERE id=?5",
        params![aluno_id, prova_id, descricao, valor, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_nota(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notas WHERE id=?1", params![id]).map_err(map_db_err)?;
    Ok(())
}
