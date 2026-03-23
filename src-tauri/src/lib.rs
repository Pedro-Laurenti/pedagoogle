mod db;
mod models;
mod dashboard;
mod professores;
mod materias;
mod alunos;
mod turmas;
mod provas;
mod notas;
mod cronograma;
mod html_render;
mod typst_pdf;
mod word;
mod configuracoes;
mod frequencia;

use dashboard::*;
use professores::*;
use materias::*;
use alunos::*;
use turmas::*;
use provas::*;
use notas::*;
use cronograma::*;
use typst_pdf::{export_prova_pdf, export_gabarito_pdf, export_boletim_pdf, export_prova_pdf_embaralhada};
use word::*;
use configuracoes::*;
use frequencia::*;
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
            list_professores, create_professor, update_professor, delete_professor,
            list_materias, create_materia, update_materia, delete_materia,
            list_alunos, create_aluno, update_aluno, delete_aluno,
            preview_import_alunos_csv, confirm_import_alunos,
            list_turmas, create_turma, update_turma, delete_turma,
            list_provas, get_prova, create_prova, update_prova, delete_prova, duplicate_prova,
            list_questoes, replace_questoes,
            list_banco_questoes, create_banco_questao, update_banco_questao, delete_banco_questao, import_from_banco,
            list_notas, create_nota, update_nota, delete_nota,
            list_aulas, create_aula, update_aula, delete_aula, copy_semestre,
            export_prova_pdf, export_prova_word, export_gabarito_pdf, export_boletim_pdf, export_prova_pdf_embaralhada,
            get_configuracoes, save_configuracoes, backup_database, restore_database,
            list_presencas, upsert_presenca, get_frequencia_aluno,
            get_dashboard_stats, list_proximas_provas, get_alertas, get_medias_por_materia,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
