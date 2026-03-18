use rusqlite::params;
use crate::db::get_conn;
use crate::models::Configuracoes;

#[tauri::command]
pub fn get_configuracoes() -> Result<Configuracoes, String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT nome_escola, logo_path, cidade, diretor FROM configuracoes WHERE id=1",
        [],
        |r| Ok(Configuracoes {
            nome_escola: r.get(0)?, logo_path: r.get(1)?,
            cidade: r.get(2)?, diretor: r.get(3)?,
        }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_configuracoes(nome_escola: String, logo_path: String, cidade: String, diretor: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE configuracoes SET nome_escola=?1, logo_path=?2, cidade=?3, diretor=?4 WHERE id=1",
        params![nome_escola, logo_path, cidade, diretor],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
