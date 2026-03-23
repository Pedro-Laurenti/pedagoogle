---
name: db-arquitetura
description: Migrações numeradas, updated_at, backup automático, paginação, Mutex e logs
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Sistema de migrações numeradas (🔴)

`src-tauri/src/db.rs`:

1. Criar tabela de controle:
```sql
CREATE TABLE IF NOT EXISTS _migrations (
    version    INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

2. Definir array de migrações:
```rust
const MIGRATIONS: &[(u32, &str)] = &[
    (1, "CREATE TABLE IF NOT EXISTS turmas (...)"),
    (2, "ALTER TABLE alunos ADD COLUMN turma_id ..."),
    // ... todas as migrações históricas em ordem
];
```

3. Função `run_migrations(conn: &Connection) -> Result<(), rusqlite::Error>`:
```rust
for (version, sql) in MIGRATIONS {
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM _migrations WHERE version = ?1", [version], |r| r.get(0)
    ).unwrap_or(0) > 0;
    if !exists {
        conn.execute_batch(sql)?;
        conn.execute("INSERT INTO _migrations (version) VALUES (?1)", [version])?;
    }
}
```

4. Substituir o bloco atual de `ALTER TABLE ... ADD COLUMN` por chamadas a `run_migrations`.
5. O bloco de criação das tabelas principais (`CREATE TABLE IF NOT EXISTS provas`, `questoes`, etc.) deve ser a migração 0 ou executado antes das migrações numeradas.

## 2. `updated_at` nas tabelas principais (🟡)

Adicionar como migrações numeradas em `MIGRATIONS`:
```sql
-- migração N
ALTER TABLE provas ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
-- migração N+1
ALTER TABLE notas  ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
-- migração N+2
ALTER TABLE alunos ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
```

Em `provas.rs`, `notas.rs`, `alunos.rs`: incluir `updated_at = datetime('now')` nos comandos UPDATE e retornar o campo nas structs/SELECTs

## 3. Backup automático na inicialização (🟡)

`src-tauri/src/db.rs` — função `backup_automatico(db_path: &str)`:
```rust
pub fn backup_automatico(db_path: &str) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string(); // requer crate chrono
    let backup_path = format!("{}.backup_{}.db", db_path.trim_end_matches(".db"), today);
    if !std::path::Path::new(&backup_path).exists() {
        let _ = std::fs::copy(db_path, &backup_path);
    }
    // limpar backups com mais de 7 dias
    // ...listar arquivos *.backup_*.db e remover os antigos
}
```
Chamar `backup_automatico(&db_path)` em `db::init()` antes de `run_migrations()`.
Adicionar crate `chrono = { version = "0.4", features = ["clock"] }` em `Cargo.toml` se não presente.

## 4. Conexão global com Mutex (🟡)

`src-tauri/src/db.rs`:
```rust
use std::sync::Mutex;
pub type DbState = Mutex<rusqlite::Connection>;

pub fn open_managed() -> DbState {
    Mutex::new(get_conn().expect("Falha ao abrir banco"))
}
```

`src-tauri/src/lib.rs`:
```rust
.manage(db::open_managed())
```

Atualizar todos os comandos para aceitar `state: tauri::State<'_, db::DbState>` e usar `state.lock().unwrap()` em vez de `db::get_conn()?`:
```rust
pub fn list_alunos(state: tauri::State<'_, db::DbState>) -> Result<Vec<Aluno>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    // ...
}
```
Aplicar progressivamente, começando pelos módulos mais usados (`alunos.rs`, `provas.rs`, `notas.rs`).

## 5. Paginação nas listagens (🟡)

**Backend** — adicionar variantes paginadas nos módulos com listas longas:
- `alunos.rs`: `list_alunos_page(page: i64, per_page: i64) -> Result<Vec<Aluno>, String>` com `LIMIT ?2 OFFSET (?1-1)*?2`
- `provas.rs`: idem `list_provas_page`
- `provas.rs`: `list_banco_questoes_page(page, per_page, filtro: String)` com LIKE no enunciado

**Frontend** — componente `src/components/Pagination.tsx`:
```tsx
// props: page, total, perPage, onChange
// renderizar: << < [páginas] > >>  usando DaisyUI join + btn
```
Usar em `alunos/page.tsx` e na aba banco de questões

## 6. Logs estruturados (🟢)

`src-tauri/Cargo.toml`:
```toml
log = "0.4"
env_logger = "0.11"
```

`src-tauri/src/lib.rs`: chamar `env_logger::init()` no início do `setup` ou `run`

Substituir todos os `eprintln!` por `log::error!` nos módulos Rust. Adicionar `log::info!("Criado: {:?}", id)` nos comandos `create_*` e `log::info!("Excluído id={}", id)` nos `delete_*`
