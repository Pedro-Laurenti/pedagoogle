use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::OnceLock;

pub static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init(path: PathBuf) {
    DB_PATH.set(path).ok();
}

pub fn get_conn() -> Result<Connection> {
    Connection::open(DB_PATH.get().expect("db not initialized"))
}

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS configuracoes (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            nome_escola TEXT NOT NULL DEFAULT '',
            logo_path TEXT NOT NULL DEFAULT '',
            cidade TEXT NOT NULL DEFAULT '',
            diretor TEXT NOT NULL DEFAULT ''
        );
        INSERT OR IGNORE INTO configuracoes (id) VALUES (1);
        CREATE TABLE IF NOT EXISTS materias (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL,
            descricao TEXT NOT NULL DEFAULT '',
            professor TEXT NOT NULL DEFAULT ''
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
    ")?;
    let additions = [
        "ALTER TABLE materias ADD COLUMN professor TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE provas ADD COLUMN valor_total REAL NOT NULL DEFAULT 10.0",
        "ALTER TABLE provas ADD COLUMN rodape TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE questoes ADD COLUMN valor REAL NOT NULL DEFAULT 0.0",
        "ALTER TABLE questoes ADD COLUMN linhas_resposta INTEGER NOT NULL DEFAULT 3",
    ];
    for sql in &additions {
        let _ = conn.execute(sql, []);
    }
    Ok(())
}
