---
name: configuracoes-extra
description: Nota mínima, ano letivo, tamanho fonte, cabeçalho por prova, backup e tema
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Nota mínima de aprovação (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE configuracoes ADD COLUMN nota_minima REAL NOT NULL DEFAULT 5.0;
```

**Backend** (`configuracoes.rs`): incluir `nota_minima: f64` na struct `Configuracoes` e nos comandos `get_configuracoes` / `save_configuracoes`

**Frontend** (`configuracoes/page.tsx`): input numérico `min="0" max="10" step="0.5"` "Nota mínima de aprovação"

**Uso**: `typst_pdf.rs` → `export_boletim_pdf` (ver `notas-extra.prompt.md`) usa `configuracoes.nota_minima` para determinar situação do aluno

## 2. Ano letivo atual (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE configuracoes ADD COLUMN ano_letivo TEXT NOT NULL DEFAULT '2026';
```

**Backend** (`configuracoes.rs`): incluir `ano_letivo: String`

**Frontend** (`configuracoes/page.tsx`): input texto "Ano letivo" (ex: 2026)

**Uso**: filtro padrão no select de turmas (filtrar por `ano_letivo` correspondente)

## 3. Tamanho de fonte no PDF (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE configuracoes ADD COLUMN tamanho_fonte INTEGER NOT NULL DEFAULT 11;
```

**Backend** (`configuracoes.rs`): incluir `tamanho_fonte: i64`

**Backend** (`typst_pdf.rs`): substituir o valor fixo em `#set text(size: ...)` pelo valor `configuracoes.tamanho_fonte` como `{tamanho_fonte}pt`

**Frontend** (`configuracoes/page.tsx`): input numérico `min="8" max="16"` "Tamanho da fonte PDF (pt)"

## 4. Cabeçalho personalizado por prova (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN escola_override TEXT NOT NULL DEFAULT '';
ALTER TABLE provas ADD COLUMN cidade_override TEXT NOT NULL DEFAULT '';
```

**Backend** (`provas.rs`): incluir os dois campos em create/update/list/get

**Backend** (`typst_pdf.rs`): ao montar o cabeçalho, usar `escola_override` se não vazio (caso contrário usar `configuracoes.escola`); idem `cidade_override`

**Frontend** (`provas/ProvaEditor.tsx`): seção colapsável `<details>` "Cabeçalho personalizado" com inputs "Escola" e "Cidade"

## 5. Backup e restauração do banco (🟢)

**Backend** (`configuracoes.rs`) — dois novos comandos:
```rust
pub fn backup_database(path: String) -> Result<(), String>
// std::fs::copy(db_path, path)

pub fn restore_database(path: String) -> Result<(), String>
// std::fs::copy(path, db_path) — atenção: fechar conexão antes de copiar ou usar hot backup via SQLite VACUUM INTO
```
O `db_path` pode ser obtido das variáveis de ambiente ou pelo mesmo mecanismo que `get_conn()` usa.

**Frontend** (`configuracoes/page.tsx`):
- Botão "Exportar backup" → `dialog.save({ filters: [{ name: "SQLite", extensions: ["db"] }] })` → chama `backup_database`
- Botão "Restaurar backup" → `dialog.open({ filters: [{ name: "SQLite", extensions: ["db"] }] })` → confirmar com `window.confirm` → chama `restore_database`

## 6. Tema padrão persistido (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE configuracoes ADD COLUMN tema TEXT NOT NULL DEFAULT 'light';
```

**Backend** (`configuracoes.rs`): incluir `tema: String`

**Frontend**:
- `configuracoes/page.tsx`: ao carregar, aplicar `document.documentElement.setAttribute("data-theme", config.tema)`; select "Tema" (light/dark) ao salvar persiste no banco
- `hooks/useTheme.ts`: priorizar valor das configurações sobre `localStorage` quando disponível
