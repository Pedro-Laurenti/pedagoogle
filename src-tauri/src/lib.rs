mod db;
mod models;
mod materias;
mod alunos;
mod provas;
mod notas;
mod cronograma;
mod html_render;
mod typst_pdf;
mod word;
mod configuracoes;

use materias::*;
use alunos::*;
use provas::*;
use notas::*;
use cronograma::*;
use typst_pdf::export_prova_pdf;
use word::*;
use configuracoes::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&data_dir)?;
            db::init(data_dir.join("pedagoogle.db"));
            let conn = db::get_conn().expect("db connection failed");
            db::migrate(&conn).expect("migration failed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_materias, create_materia, update_materia, delete_materia,
            list_alunos, create_aluno, update_aluno, delete_aluno,
            list_provas, get_prova, create_prova, update_prova, delete_prova,
            list_questoes, replace_questoes,
            list_notas, create_nota, update_nota, delete_nota,
            list_aulas, create_aula, delete_aula,
            export_prova_pdf, export_prova_word,
            get_configuracoes, save_configuracoes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
