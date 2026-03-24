use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use chrono::TimeZone;

pub static DB_PATH: OnceLock<PathBuf> = OnceLock::new();
pub type DbState = Mutex<Connection>;

pub fn init(path: PathBuf) {
    DB_PATH.set(path).ok();
}

pub fn get_conn() -> Result<Connection> {
    Connection::open(DB_PATH.get().expect("db not initialized"))
}

pub fn open_managed() -> DbState {
    Mutex::new(get_conn().expect("Falha ao abrir banco"))
}

const SCHEMA: &str = "
    PRAGMA journal_mode=WAL;
    CREATE TABLE IF NOT EXISTS configuracoes (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        nome_escola TEXT NOT NULL DEFAULT '',
        logo_path TEXT NOT NULL DEFAULT '',
        cidade TEXT NOT NULL DEFAULT '',
        diretor TEXT NOT NULL DEFAULT ''
    );
    INSERT OR IGNORE INTO configuracoes (id) VALUES (1);
    CREATE TABLE IF NOT EXISTS professores (
        id    INTEGER PRIMARY KEY AUTOINCREMENT,
        nome  TEXT NOT NULL,
        email TEXT NOT NULL DEFAULT ''
    );
    CREATE TABLE IF NOT EXISTS materias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nome TEXT NOT NULL,
        descricao TEXT NOT NULL DEFAULT '',
        professor TEXT NOT NULL DEFAULT ''
    );
    CREATE TABLE IF NOT EXISTS turmas (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        nome       TEXT NOT NULL,
        ano_letivo TEXT NOT NULL DEFAULT '2026',
        turno      TEXT NOT NULL DEFAULT 'Manhã'
    );
    CREATE TABLE IF NOT EXISTS alunos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nome TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS provas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        titulo TEXT NOT NULL,
        descricao TEXT NOT NULL DEFAULT '',
        materia_id INTEGER REFERENCES materias(id) ON DELETE SET NULL,
        data TEXT NOT NULL DEFAULT '',
        rodape TEXT NOT NULL DEFAULT '',
        margens TEXT NOT NULL DEFAULT 'normal',
        valor_total REAL NOT NULL DEFAULT 10.0
    );
    CREATE TABLE IF NOT EXISTS questoes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        prova_id INTEGER NOT NULL REFERENCES provas(id) ON DELETE CASCADE,
        enunciado TEXT NOT NULL,
        tipo TEXT NOT NULL,
        opcoes TEXT NOT NULL DEFAULT '[]',
        ordem INTEGER NOT NULL DEFAULT 0,
        valor REAL NOT NULL DEFAULT 0.0,
        linhas_resposta INTEGER NOT NULL DEFAULT 3
    );
    CREATE TABLE IF NOT EXISTS notas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        aluno_id INTEGER NOT NULL REFERENCES alunos(id) ON DELETE CASCADE,
        prova_id INTEGER REFERENCES provas(id) ON DELETE SET NULL,
        descricao TEXT NOT NULL DEFAULT '',
        valor REAL NOT NULL
    );
    CREATE TABLE IF NOT EXISTS aulas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        materia_id INTEGER REFERENCES materias(id) ON DELETE SET NULL,
        dia_semana TEXT NOT NULL,
        hora_inicio TEXT NOT NULL,
        hora_fim TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS banco_questoes (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        tipo        TEXT NOT NULL DEFAULT 'dissertativa',
        enunciado   TEXT NOT NULL DEFAULT '',
        opcoes      TEXT NOT NULL DEFAULT '[]',
        valor       REAL NOT NULL DEFAULT 1.0,
        tags        TEXT NOT NULL DEFAULT '',
        dificuldade TEXT NOT NULL DEFAULT 'médio'
    );
    CREATE INDEX IF NOT EXISTS idx_questoes_prova ON questoes(prova_id);
    CREATE INDEX IF NOT EXISTS idx_notas_aluno ON notas(aluno_id);
    CREATE INDEX IF NOT EXISTS idx_notas_prova ON notas(prova_id);
    CREATE INDEX IF NOT EXISTS idx_aulas_dia ON aulas(dia_semana);
";

const MIGRATIONS: &[(u32, &str)] = &[
    (1,  "ALTER TABLE configuracoes ADD COLUMN moldura_estilo TEXT NOT NULL DEFAULT 'none'"),
    (2,  "ALTER TABLE configuracoes ADD COLUMN margem_folha REAL NOT NULL DEFAULT 15.0"),
    (3,  "ALTER TABLE configuracoes ADD COLUMN margem_moldura REAL NOT NULL DEFAULT 5.0"),
    (4,  "ALTER TABLE configuracoes ADD COLUMN margem_conteudo REAL NOT NULL DEFAULT 5.0"),
    (5,  "ALTER TABLE alunos ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL"),
    (6,  "ALTER TABLE alunos ADD COLUMN matricula TEXT NOT NULL DEFAULT ''"),
    (7,  "ALTER TABLE configuracoes ADD COLUMN fonte TEXT NOT NULL DEFAULT 'New Computer Modern'"),
    (8,  "ALTER TABLE materias ADD COLUMN professor_id INTEGER REFERENCES professores(id) ON DELETE SET NULL"),
    (9,  "ALTER TABLE alunos ADD COLUMN foto_path TEXT NOT NULL DEFAULT ''"),
    (10, "ALTER TABLE materias ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL"),
    (11, "ALTER TABLE materias ADD COLUMN carga_horaria_semanal INTEGER NOT NULL DEFAULT 0"),
    (12, "ALTER TABLE materias ADD COLUMN cor TEXT NOT NULL DEFAULT '#6366f1'"),
    (13, "ALTER TABLE configuracoes ADD COLUMN nota_minima REAL NOT NULL DEFAULT 5.0"),
    (14, "ALTER TABLE configuracoes ADD COLUMN ano_letivo TEXT NOT NULL DEFAULT '2026'"),
    (15, "ALTER TABLE configuracoes ADD COLUMN tamanho_fonte INTEGER NOT NULL DEFAULT 11"),
    (16, "ALTER TABLE configuracoes ADD COLUMN tema TEXT NOT NULL DEFAULT 'light'"),
    (17, "ALTER TABLE provas ADD COLUMN escola_override TEXT NOT NULL DEFAULT ''"),
    (18, "ALTER TABLE provas ADD COLUMN cidade_override TEXT NOT NULL DEFAULT ''"),
    (19, "ALTER TABLE aulas ADD COLUMN semestre TEXT NOT NULL DEFAULT '2026-1'"),
    (20, "ALTER TABLE provas ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL"),
    (21, "ALTER TABLE notas ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''"),
    (22, "ALTER TABLE provas ADD COLUMN is_recuperacao INTEGER NOT NULL DEFAULT 0"),
    (23, "ALTER TABLE provas ADD COLUMN qr_gabarito INTEGER NOT NULL DEFAULT 0"),
    (24, "CREATE TABLE IF NOT EXISTS presencas (id INTEGER PRIMARY KEY AUTOINCREMENT, aluno_id INTEGER NOT NULL REFERENCES alunos(id) ON DELETE CASCADE, aula_id INTEGER NOT NULL REFERENCES aulas(id) ON DELETE CASCADE, data TEXT NOT NULL, presente INTEGER NOT NULL DEFAULT 1, UNIQUE(aluno_id, aula_id))"),
    (25, "ALTER TABLE questoes ADD COLUMN tags TEXT NOT NULL DEFAULT ''"),
    (26, "ALTER TABLE questoes ADD COLUMN dificuldade TEXT NOT NULL DEFAULT 'médio'"),
    (27, "ALTER TABLE provas ADD COLUMN duas_colunas INTEGER NOT NULL DEFAULT 0"),
    (28, "ALTER TABLE provas ADD COLUMN paisagem INTEGER NOT NULL DEFAULT 0"),
    (29, "ALTER TABLE provas ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''"),
    (30, "ALTER TABLE alunos ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''"),
    (31, "ALTER TABLE professores ADD COLUMN telefone TEXT NOT NULL DEFAULT ''"),
    (32, "ALTER TABLE professores ADD COLUMN especialidade TEXT NOT NULL DEFAULT ''"),
    (33, "ALTER TABLE professores ADD COLUMN aulas_por_semana INTEGER NOT NULL DEFAULT 0"),
    (34, "ALTER TABLE professores ADD COLUMN observacoes TEXT NOT NULL DEFAULT ''"),
    (35, "CREATE TABLE IF NOT EXISTS professor_materias (professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE, materia_id INTEGER NOT NULL REFERENCES materias(id) ON DELETE CASCADE, PRIMARY KEY (professor_id, materia_id))"),
    (36, "CREATE TABLE IF NOT EXISTS professor_turmas (professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE, turma_id INTEGER NOT NULL REFERENCES turmas(id) ON DELETE CASCADE, PRIMARY KEY (professor_id, turma_id))"),
    (37, "CREATE TABLE IF NOT EXISTS professor_cronograma (id INTEGER PRIMARY KEY AUTOINCREMENT, professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE, titulo TEXT NOT NULL, dia_semana INTEGER NOT NULL, hora_inicio TEXT NOT NULL, hora_fim TEXT NOT NULL, cor TEXT NOT NULL DEFAULT '#3b82f6', recorrente INTEGER NOT NULL DEFAULT 1)"),
    (38, "ALTER TABLE materias ADD COLUMN icone TEXT NOT NULL DEFAULT 'MdBook'"),
    (39, "ALTER TABLE configuracoes ADD COLUMN usar_turmas INTEGER NOT NULL DEFAULT 1"),
    (40, "ALTER TABLE configuracoes ADD COLUMN usar_professores INTEGER NOT NULL DEFAULT 1"),
    (41, "ALTER TABLE configuracoes ADD COLUMN usar_frequencia INTEGER NOT NULL DEFAULT 1"),
    (42, "ALTER TABLE configuracoes ADD COLUMN usar_recuperacao INTEGER NOT NULL DEFAULT 1"),
    (43, "ALTER TABLE aulas ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL"),
    (44, "ALTER TABLE aulas ADD COLUMN aluno_ids TEXT NOT NULL DEFAULT '[]'"),
    (45, "CREATE TABLE IF NOT EXISTS categoria_lancamentos (id INTEGER PRIMARY KEY AUTOINCREMENT, nome TEXT NOT NULL, cor TEXT NOT NULL DEFAULT '#6366f1'); INSERT OR IGNORE INTO categoria_lancamentos (id, nome, cor) VALUES (1, 'Prova', '#6366f1'), (2, 'Trabalho', '#10b981'), (3, 'Participação', '#f59e0b');"),
    (46, "ALTER TABLE notas ADD COLUMN categoria_id INTEGER REFERENCES categoria_lancamentos(id) ON DELETE SET NULL"),
    (47, "ALTER TABLE aulas ADD COLUMN bimestre INTEGER NOT NULL DEFAULT 1"),
    (48, "ALTER TABLE configuracoes ADD COLUMN aulas_por_dia INTEGER NOT NULL DEFAULT 6"),
    (49, "ALTER TABLE configuracoes ADD COLUMN minutos_por_aula INTEGER NOT NULL DEFAULT 45"),
    (50, "ALTER TABLE configuracoes ADD COLUMN hora_entrada TEXT NOT NULL DEFAULT '07:00'"),
    (51, "ALTER TABLE configuracoes ADD COLUMN dias_letivos_semana INTEGER NOT NULL DEFAULT 5"),
    (52, "ALTER TABLE turmas ADD COLUMN ano TEXT NOT NULL DEFAULT ''"),
    (53, "ALTER TABLE turmas ADD COLUMN turma TEXT NOT NULL DEFAULT ''"),
    (54, "CREATE TABLE IF NOT EXISTS turma_materias (turma_id INTEGER NOT NULL REFERENCES turmas(id) ON DELETE CASCADE, materia_id INTEGER NOT NULL REFERENCES materias(id) ON DELETE CASCADE, PRIMARY KEY (turma_id, materia_id))"),
    (55, "CREATE TABLE IF NOT EXISTS aluno_materias (aluno_id INTEGER NOT NULL REFERENCES alunos(id) ON DELETE CASCADE, materia_id INTEGER NOT NULL REFERENCES materias(id) ON DELETE CASCADE, PRIMARY KEY (aluno_id, materia_id))"),
];

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version    INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;
    for (version, sql) in MIGRATIONS {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
            [version],
            |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if !exists {
            match conn.execute_batch(sql) {
                Ok(_) => {}
                Err(e) => {
                    let msg = e.to_string();
                    if !msg.contains("duplicate column name") && !msg.contains("already exists") {
                        return Err(e);
                    }
                }
            }
            conn.execute("INSERT INTO _migrations (version) VALUES (?1)", [version])?;
        }
    }
    Ok(())
}

pub fn backup_automatico(db_path: &str) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let backup_path = format!("{}.backup_{}.db", db_path.trim_end_matches(".db"), today);
    if !std::path::Path::new(&backup_path).exists() {
        let _ = std::fs::copy(db_path, &backup_path);
    }
    let base = db_path.trim_end_matches(".db");
    if let Ok(entries) = std::fs::read_dir(std::path::Path::new(db_path).parent().unwrap_or(std::path::Path::new("."))) {
        let cutoff = chrono::Local::now() - chrono::Duration::days(7);
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(std::path::Path::new(base).file_name().unwrap_or_default().to_string_lossy().as_ref())
                && name.contains(".backup_")
                && name.ends_with(".db")
            {
                if let Some(date_str) = name.split(".backup_").nth(1).and_then(|s| s.strip_suffix(".db")) {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        let dt = date.and_hms_opt(0, 0, 0).map(|dt| chrono::Local.from_local_datetime(&dt).single());
                        if let Some(Some(dt)) = dt {
                            if dt < cutoff {
                                let _ = std::fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    run_migrations(conn)
}
