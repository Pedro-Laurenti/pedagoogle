---
description: Requisitos para implementar novo módulo ou comando no backend (Tauri/Rust).
applyTo: "src-tauri/src/**/*.rs"
---

# Novo Módulo Backend

## Estrutura
1. Criar `src-tauri/src/[modulo].rs`
2. Em `src-tauri/src/lib.rs`:
   ```rust
   mod [modulo];
   use [modulo]::*;
   // adicionar novos comandos em invoke_handler![...]
   ```

## Padrão de comando
```rust
#[tauri::command]
pub fn create_item(nome: String) -> Result<i64, String> {
    let conn = db::get_conn().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO items (nome) VALUES (?1)", [&nome])
        .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
```

## Regras
- Retornar sempre `Result<T, String>`, erros via `.map_err(|e| e.to_string())`
- Args camelCase no TypeScript → Tauri converte para snake_case em Rust automaticamente
- Nomes de comandos: snake_case (ex: `list_alunos`, `create_aluno`)

## Novos modelos
`src-tauri/src/models.rs`:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub nome: String,
}
```

## Novas tabelas/colunas
`src-tauri/src/db.rs` → função `migrate()`:
```sql
CREATE TABLE IF NOT EXISTS items (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    nome TEXT NOT NULL
);
-- para colunas em tabelas existentes:
ALTER TABLE items ADD COLUMN campo TEXT NOT NULL DEFAULT '';
```

## Módulos existentes
| Arquivo | Entidade |
|--|--|
| `alunos.rs` | CRUD de alunos |
| `materias.rs` | CRUD de matérias |
| `provas.rs` | provas + questões |
| `notas.rs` | notas |
| `cronograma.rs` | aulas |
| `configuracoes.rs` | configurações globais |
| `typst_pdf.rs` | exportação PDF (Typst) |
| `word.rs` | exportação Word |
| `db.rs` | init, `get_conn()`, `migrate()` |
| `models.rs` | structs Serde |
