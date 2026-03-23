use rusqlite::params;
use crate::db::{get_conn, DB_PATH};
use crate::models::Configuracoes;

#[tauri::command]
pub fn get_configuracoes() -> Result<Configuracoes, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT nome_escola, logo_path, cidade, diretor,
                COALESCE(moldura_estilo, 'none'), COALESCE(margem_folha, 15.0),
                COALESCE(margem_moldura, 5.0), COALESCE(margem_conteudo, 5.0),
                COALESCE(fonte, 'New Computer Modern'),
                COALESCE(nota_minima, 5.0), COALESCE(ano_letivo, '2026'),
                COALESCE(tamanho_fonte, 11), COALESCE(tema, 'light'),
                COALESCE(usar_turmas, 1), COALESCE(usar_professores, 1),
                COALESCE(usar_frequencia, 1), COALESCE(usar_recuperacao, 1),
                COALESCE(aulas_por_dia, 6), COALESCE(minutos_por_aula, 45),
                COALESCE(hora_entrada, '07:00'), COALESCE(dias_letivos_semana, 5)
         FROM configuracoes WHERE id=1",
        [],
        |r| Ok(Configuracoes {
            nome_escola: r.get(0)?, logo_path: r.get(1)?,
            cidade: r.get(2)?, diretor: r.get(3)?,
            moldura_estilo: r.get(4)?, margem_folha: r.get(5)?,
            margem_moldura: r.get(6)?, margem_conteudo: r.get(7)?,
            fonte: r.get(8)?,
            nota_minima: r.get(9)?, ano_letivo: r.get(10)?,
            tamanho_fonte: r.get(11)?, tema: r.get(12)?,
            usar_turmas: r.get::<_, i64>(13)? != 0,
            usar_professores: r.get::<_, i64>(14)? != 0,
            usar_frequencia: r.get::<_, i64>(15)? != 0,
            usar_recuperacao: r.get::<_, i64>(16)? != 0,
            aulas_por_dia: r.get(17)?,
            minutos_por_aula: r.get(18)?,
            hora_entrada: r.get(19)?,
            dias_letivos_semana: r.get(20)?,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_configuracoes(
    nome_escola: String, logo_path: String, cidade: String, diretor: String,
    moldura_estilo: String, margem_folha: f64, margem_moldura: f64, margem_conteudo: f64,
    fonte: String, nota_minima: f64, ano_letivo: String, tamanho_fonte: i64, tema: String,
    usar_turmas: bool, usar_professores: bool, usar_frequencia: bool, usar_recuperacao: bool,
    aulas_por_dia: i64, minutos_por_aula: i64, hora_entrada: String, dias_letivos_semana: i64,
) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE configuracoes SET nome_escola=?1, logo_path=?2, cidade=?3, diretor=?4,
         moldura_estilo=?5, margem_folha=?6, margem_moldura=?7, margem_conteudo=?8,
         fonte=?9, nota_minima=?10, ano_letivo=?11, tamanho_fonte=?12, tema=?13,
         usar_turmas=?14, usar_professores=?15, usar_frequencia=?16, usar_recuperacao=?17,
         aulas_por_dia=?18, minutos_por_aula=?19, hora_entrada=?20, dias_letivos_semana=?21
         WHERE id=1",
        params![nome_escola, logo_path, cidade, diretor, moldura_estilo, margem_folha,
                margem_moldura, margem_conteudo, fonte, nota_minima, ano_letivo, tamanho_fonte, tema,
                usar_turmas as i64, usar_professores as i64, usar_frequencia as i64, usar_recuperacao as i64,
                aulas_por_dia, minutos_por_aula, hora_entrada, dias_letivos_semana],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn check_update() -> Result<Option<String>, String> {
    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent(concat!("pedagoogle/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get("https://github.com/Pedro-Laurenti/pedagoogle/releases/latest")
        .send()
        .map_err(|e| e.to_string())?;
    let final_url = response.url().to_string();
    let tag = final_url
        .split("/tag/")
        .nth(1)
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    if tag.is_empty() {
        return Ok(None);
    }
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if tag != current { Ok(Some(tag)) } else { Ok(None) }
}

#[tauri::command]
pub fn backup_database(path: String) -> Result<(), String> {
    let db_path = DB_PATH.get().ok_or("DB não inicializado")?;
    std::fs::copy(db_path, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn restore_database(path: String) -> Result<(), String> {
    let db_path = DB_PATH.get().ok_or("DB não inicializado")?;
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute_batch(&format!("VACUUM INTO '{}'", path.replace('\'', "''")))
        .ok();
    drop(conn);
    std::fs::copy(&path, db_path).map_err(|e| e.to_string())?;
    Ok(())
}

