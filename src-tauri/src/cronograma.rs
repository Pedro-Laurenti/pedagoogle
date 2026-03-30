use rusqlite::params;
use crate::db::get_conn;
use crate::models::*;
use serde_json;

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

fn time_to_mins(t: &str) -> i64 {
    let parts: Vec<i64> = t.splitn(2, ':').map(|p| p.parse().unwrap_or(0)).collect();
    parts[0] * 60 + parts.get(1).copied().unwrap_or(0)
}

fn get_schedule_bounds(conn: &rusqlite::Connection) -> Result<(i64, i64), String> {
    let (hora_entrada, aulas_por_dia, minutos_por_aula): (String, i64, i64) = conn.query_row(
        "SELECT COALESCE(hora_entrada,'07:00'), COALESCE(aulas_por_dia,6), COALESCE(minutos_por_aula,45) FROM configuracoes WHERE id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| e.to_string())?;
    let start = time_to_mins(&hora_entrada);
    Ok((start, start + aulas_por_dia * minutos_por_aula))
}

fn check_horario(conn: &rusqlite::Connection, hora_inicio: &str, hora_fim: &str) -> Result<(), String> {
    let (schedule_start, schedule_end) = get_schedule_bounds(conn)?;
    let inicio = time_to_mins(hora_inicio);
    let fim = time_to_mins(hora_fim);
    if inicio >= fim {
        return Err("Horário de início deve ser anterior ao horário de fim.".into());
    }
    if inicio < schedule_start || fim > schedule_end {
        return Err("Horário fora do período letivo configurado.".into());
    }
    Ok(())
}

fn check_conflito(conn: &rusqlite::Connection, dia_semana: &str, hora_inicio: &str, hora_fim: &str, semestre: &str, excluir_id: i64) -> Result<(), String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM aulas WHERE dia_semana=?1 AND id!=?2 AND hora_inicio<?3 AND hora_fim>?4 AND semestre=?5",
        params![dia_semana, excluir_id, hora_fim, hora_inicio, semestre],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    if count > 0 { return Err("Conflito de horário com outra aula.".into()); }
    Ok(())
}

#[tauri::command]
pub fn list_aulas(semestre: Option<String>) -> Result<Vec<Aula>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, COALESCE(aluno_ids,'[]'), COALESCE(bimestre,1) FROM aulas ORDER BY dia_semana, hora_inicio"
    ).map_err(|e| e.to_string())?;
    let all: Vec<Aula> = stmt.query_map([], |r| Ok(Aula {
        id: r.get(0)?, materia_id: r.get(1)?, dia_semana: r.get(2)?,
        hora_inicio: r.get(3)?, hora_fim: r.get(4)?, semestre: r.get(5)?,
        turma_id: r.get(6)?, aluno_ids: r.get(7)?, bimestre: r.get(8)?,
    })).map_err(|e| e.to_string())?
       .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(match semestre {
        Some(s) => all.into_iter().filter(|a| a.semestre == s).collect(),
        None => all,
    })
}

#[tauri::command]
pub fn create_aula(materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String, turma_id: Option<i64>, aluno_ids: Option<String>) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_horario(&conn, &hora_inicio, &hora_fim)?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, 0)?;
    let aids = aluno_ids.unwrap_or_else(|| "[]".into());
    conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aluno_ids, bimestre) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aids],
    ).map_err(map_db_err)?;
    let id = conn.last_insert_rowid();
    log::info!("Criado: aula id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn create_aulas_recorrentes(materia_id: Option<i64>, dias_semana: Vec<String>, hora_inicio: String, hora_fim: String, semestre: String, turma_id: Option<i64>, aluno_ids: Option<String>) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_horario(&conn, &hora_inicio, &hora_fim)?;
    let aids = aluno_ids.unwrap_or_else(|| "[]".into());
    let mut count = 0i64;
    for dia in &dias_semana {
        check_conflito(&conn, dia, &hora_inicio, &hora_fim, &semestre, 0)?;
        conn.execute(
            "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aluno_ids, bimestre) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
            params![materia_id, dia, hora_inicio, hora_fim, semestre, turma_id, aids],
        ).map_err(map_db_err)?;
        count += 1;
    }
    log::info!("Criadas {} aulas recorrentes", count);
    Ok(count)
}

#[tauri::command]
pub fn update_aula(id: i64, materia_id: Option<i64>, dia_semana: String, hora_inicio: String, hora_fim: String, semestre: String, turma_id: Option<i64>, aluno_ids: Option<String>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    check_horario(&conn, &hora_inicio, &hora_fim)?;
    check_conflito(&conn, &dia_semana, &hora_inicio, &hora_fim, &semestre, id)?;
    let aids = aluno_ids.unwrap_or_else(|| "[]".into());
    conn.execute(
        "UPDATE aulas SET materia_id=?1, dia_semana=?2, hora_inicio=?3, hora_fim=?4, semestre=?5, turma_id=?6, aluno_ids=?7 WHERE id=?8",
        params![materia_id, dia_semana, hora_inicio, hora_fim, semestre, turma_id, aids, id],
    ).map_err(map_db_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_aula(id: i64) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM aulas WHERE id=?1", params![id]).map_err(map_db_err)?;
    log::info!("Excluído id={} (aula)", id);
    Ok(())
}

#[tauri::command]
pub fn copy_semestre(de: String, para: String) -> Result<i64, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let count = conn.execute(
        "INSERT INTO aulas (materia_id, dia_semana, hora_inicio, hora_fim, semestre, bimestre) SELECT materia_id, dia_semana, hora_inicio, hora_fim, ?1, COALESCE(bimestre,1) FROM aulas WHERE semestre=?2",
        params![para, de],
    ).map_err(|e| e.to_string())?;
    Ok(count as i64)
}

#[tauri::command]
pub fn get_faltas_aula(aula_id: i64, data: String) -> Result<Vec<crate::models::FaltaItem>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    struct AulaInfo { aluno_ids: String, turma_id: Option<i64> }
    let aula = conn.query_row(
        "SELECT COALESCE(aluno_ids,'[]'), turma_id FROM aulas WHERE id=?1",
        params![aula_id],
        |r| Ok(AulaInfo { aluno_ids: r.get(0)?, turma_id: r.get(1)? }),
    ).map_err(|e| e.to_string())?;
    let ids: Vec<i64> = serde_json::from_str(&aula.aluno_ids).unwrap_or_default();
    let alunos: Vec<(i64, String)> = if !ids.is_empty() {
        ids.iter().filter_map(|&id| {
            conn.query_row("SELECT id, nome FROM alunos WHERE id=?1", params![id], |r| Ok((r.get(0)?, r.get(1)?))).ok()
        }).collect()
    } else if let Some(tid) = aula.turma_id {
        let mut stmt = conn.prepare("SELECT id, nome FROM alunos WHERE turma_id=?1 ORDER BY nome").map_err(|e| e.to_string())?;
        stmt.query_map(params![tid], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn.prepare("SELECT id, nome FROM alunos ORDER BY nome").map_err(|e| e.to_string())?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };
    alunos.into_iter().map(|(id, nome)| {
        let faltou = conn.query_row(
            "SELECT COUNT(*) FROM faltas WHERE aluno_id=?1 AND aula_id=?2 AND data=?3",
            params![id, aula_id, &data],
            |r| r.get::<_, i64>(0),
        ).map(|c| c > 0).unwrap_or(false);
        Ok(crate::models::FaltaItem { aluno_id: id, aluno_nome: nome, faltou })
    }).collect()
}

#[tauri::command]
pub fn save_faltas_aula(aula_id: i64, data: String, faltaram: Vec<i64>) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM faltas WHERE aula_id=?1 AND data=?2", params![aula_id, &data]).map_err(|e| e.to_string())?;
    for aluno_id in &faltaram {
        conn.execute(
            "INSERT OR IGNORE INTO faltas (aluno_id, aula_id, data) VALUES (?1, ?2, ?3)",
            params![aluno_id, aula_id, &data],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_faltas_por_materia(aluno_id: i64) -> Result<Vec<crate::models::FaltasPorMateria>, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT a.materia_id, m.nome, a.bimestre, COUNT(*) as faltas
         FROM faltas f
         JOIN aulas a ON a.id = f.aula_id
         JOIN materias m ON m.id = a.materia_id
         WHERE f.aluno_id = ?1 AND a.materia_id IS NOT NULL
         GROUP BY a.materia_id, a.bimestre
         ORDER BY m.nome, a.bimestre"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![aluno_id], |r| Ok(crate::models::FaltasPorMateria {
        materia_id: r.get(0)?,
        materia_nome: r.get(1)?,
        bimestre: r.get(2)?,
        faltas: r.get(3)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
