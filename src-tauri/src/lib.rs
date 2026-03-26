mod db;
mod models;
mod dashboard;
mod professores;
mod materias;
mod alunos;
mod turmas;
mod provas;
mod atividades;
mod notas;
mod cronograma;
mod html_render;
mod typst_pdf;
mod word;
mod configuracoes;

use dashboard::*;
use professores::*;
use materias::*;
use alunos::*;
use turmas::*;
use provas::*;
use atividades::*;
use notas::*;
use cronograma::*;
use typst_pdf::{export_prova_pdf, export_gabarito_pdf, export_boletim_pdf, export_atividade_pdf, export_gabarito_atividade_pdf};
use word::*;
use configuracoes::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("pedagoogle.db");
            db::backup_automatico(db_path.to_str().expect("path inválido"));
            db::init(db_path);
            let conn = db::get_conn().expect("db connection failed");
            db::migrate(&conn).expect("migration failed");
            app.manage(db::open_managed());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_professores, create_professor, update_professor, delete_professor,
            set_professor_materias, set_professor_turmas,
            list_professor_materias, list_professor_turmas,
            list_professor_cronograma, save_professor_cronograma,
            list_materias, create_materia, update_materia, delete_materia,
            list_alunos, create_aluno, update_aluno, delete_aluno,
            set_aluno_materias, list_aluno_materias,
            list_alunos_page,
            list_turmas, create_turma, update_turma, delete_turma,
            set_turma_materias, list_turma_materias,
            list_provas, get_prova, create_prova, update_prova, delete_prova, duplicate_prova,
            list_provas_page,
            list_questoes, replace_questoes,
            list_atividades, get_atividade, create_atividade, update_atividade, delete_atividade, duplicate_atividade,
            list_questoes_atividade, replace_questoes_atividade,
            list_notas, create_nota, update_nota, delete_nota,
            list_categoria_lancamentos, create_categoria_lancamento, update_categoria_lancamento, delete_categoria_lancamento,
            list_aulas, create_aula, create_aulas_recorrentes, update_aula, delete_aula, copy_semestre,
            export_prova_pdf, export_prova_word, export_gabarito_pdf, export_gabarito_word, export_boletim_pdf,
            export_atividade_pdf, export_gabarito_atividade_pdf, export_gabarito_atividade_word, export_atividade_word,
            get_configuracoes, save_configuracoes, backup_database, restore_database, check_update,
            get_dashboard_stats, list_proximas_provas, get_alertas, get_medias_por_materia,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
