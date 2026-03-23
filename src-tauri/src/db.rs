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
    ")?;
    let additions = [
        "ALTER TABLE materias ADD COLUMN professor TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE provas ADD COLUMN valor_total REAL NOT NULL DEFAULT 10.0",
        "ALTER TABLE provas ADD COLUMN rodape TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE questoes ADD COLUMN valor REAL NOT NULL DEFAULT 0.0",
        "ALTER TABLE questoes ADD COLUMN linhas_resposta INTEGER NOT NULL DEFAULT 3",
        // Moldura/frame settings
        "ALTER TABLE configuracoes ADD COLUMN moldura_estilo TEXT NOT NULL DEFAULT 'none'",
        "ALTER TABLE configuracoes ADD COLUMN margem_folha REAL NOT NULL DEFAULT 15.0",
        "ALTER TABLE configuracoes ADD COLUMN margem_moldura REAL NOT NULL DEFAULT 5.0",
        "ALTER TABLE configuracoes ADD COLUMN margem_conteudo REAL NOT NULL DEFAULT 5.0",
        "ALTER TABLE alunos ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL",
        "ALTER TABLE alunos ADD COLUMN matricula TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE configuracoes ADD COLUMN fonte TEXT NOT NULL DEFAULT 'New Computer Modern'",
        "ALTER TABLE materias ADD COLUMN professor_id INTEGER REFERENCES professores(id) ON DELETE SET NULL",
        "ALTER TABLE alunos ADD COLUMN foto_path TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE materias ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL",
        "ALTER TABLE materias ADD COLUMN carga_horaria_semanal INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE materias ADD COLUMN cor TEXT NOT NULL DEFAULT '#6366f1'",
        "ALTER TABLE configuracoes ADD COLUMN nota_minima REAL NOT NULL DEFAULT 5.0",
        "ALTER TABLE configuracoes ADD COLUMN ano_letivo TEXT NOT NULL DEFAULT '2026'",
        "ALTER TABLE configuracoes ADD COLUMN tamanho_fonte INTEGER NOT NULL DEFAULT 11",
        "ALTER TABLE configuracoes ADD COLUMN tema TEXT NOT NULL DEFAULT 'light'",
        "ALTER TABLE provas ADD COLUMN escola_override TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE provas ADD COLUMN cidade_override TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE aulas ADD COLUMN semestre TEXT NOT NULL DEFAULT '2026-1'",
        "ALTER TABLE provas ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL",
        "ALTER TABLE notas ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'))",
        "ALTER TABLE provas ADD COLUMN is_recuperacao INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE provas ADD COLUMN qr_gabarito INTEGER NOT NULL DEFAULT 0",
        "CREATE TABLE IF NOT EXISTS presencas (id INTEGER PRIMARY KEY AUTOINCREMENT, aluno_id INTEGER NOT NULL REFERENCES alunos(id) ON DELETE CASCADE, aula_id INTEGER NOT NULL REFERENCES aulas(id) ON DELETE CASCADE, data TEXT NOT NULL, presente INTEGER NOT NULL DEFAULT 1, UNIQUE(aluno_id, aula_id))",
        "ALTER TABLE questoes ADD COLUMN tags TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE questoes ADD COLUMN dificuldade TEXT NOT NULL DEFAULT 'médio'",
        "ALTER TABLE provas ADD COLUMN duas_colunas INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE provas ADD COLUMN paisagem INTEGER NOT NULL DEFAULT 0",
    ];
    for sql in &additions {
        let _ = conn.execute(sql, []);
    }
    Ok(())
}
