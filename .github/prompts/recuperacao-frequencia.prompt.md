---
name: recuperacao-frequencia
description: Recuperação/segunda chamada, presença/frequência e QR code de gabarito
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Recuperação / Segunda chamada (🔴)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN is_recuperacao INTEGER NOT NULL DEFAULT 0;
```

**Backend** (`provas.rs`): incluir `is_recuperacao: bool` em create/update/list/get

**Backend** (`notas.rs`) — nova função auxiliar (não exposta como comando):
```rust
pub fn get_nota_efetiva(conn: &Connection, aluno_id: i64, materia_id: i64) -> f64
```
- Buscar notas do aluno em provas da matéria
- Para provas regulares (`is_recuperacao = 0`) e de recuperação (`is_recuperacao = 1`) do mesmo aluno+matéria, retornar a maior nota
- Usar esta função no cálculo de média do boletim (`export_boletim_pdf`)

**Frontend** (`provas/ProvaEditor.tsx`): checkbox "É prova de recuperação / segunda chamada"

**Frontend** (`provas/page.tsx`): badge `REC` (badge badge-warning) na coluna de título para provas com `is_recuperacao = true`

## 2. Presença / Frequência (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
CREATE TABLE IF NOT EXISTS presencas (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    aluno_id INTEGER NOT NULL REFERENCES alunos(id) ON DELETE CASCADE,
    aula_id  INTEGER NOT NULL REFERENCES aulas(id) ON DELETE CASCADE,
    data     TEXT NOT NULL,
    presente INTEGER NOT NULL DEFAULT 1,
    UNIQUE(aluno_id, aula_id)
);
```

**Backend** — criar `src-tauri/src/frequencia.rs` com:
- `list_presencas(aula_id: i64) -> Vec<Presenca { id, aluno_id, aula_id, data, presente }>` com JOIN alunos para incluir `aluno_nome`
- `upsert_presenca(aluno_id: i64, aula_id: i64, data: String, presente: bool) -> Result<(), String>` — INSERT OR REPLACE
- `get_frequencia_aluno(aluno_id: i64) -> Vec<FrequenciaMateria { materia_nome: String, total_aulas: i64, presencas: i64, percentual: f64 }>`

Registrar os três comandos em `lib.rs` e declarar `mod frequencia;`

**Frontend** — criar `src/app/(main)/frequencia/page.tsx`:
- Select "Aula" (opções de `list_aulas`) e campo data no topo
- Ao selecionar aula: carregar lista de alunos da turma da aula + `list_presencas(aula_id)`
- Grid: linhas = alunos, checkbox "Presente" por aluno; ao alterar, chamar `upsert_presenca`
- Seção "Relatório de frequência": select de aluno, exibir `get_frequencia_aluno` em tabela
- Adicionar entrada "Frequência" no menu em `navigation.ts` (ícone: `MdFactCheck`)

**Tipos** (`types/index.ts`):
```ts
export interface Presenca { id: number; aluno_id: number; aula_id: number; aluno_nome: string; data: string; presente: boolean; }
export interface FrequenciaMateria { materia_nome: string; total_aulas: number; presencas: number; percentual: number; }
```

## 3. QR Code de gabarito no PDF (🟡)

**Backend** (`Cargo.toml`): adicionar `qrcode = "0.14"` e `image = { version = "0.25", features = ["png"] }`

**Backend** (`typst_pdf.rs`):
- Função `generate_qr_png(text: &str, tmp_path: &str) -> Result<(), String>` — gera QR code como PNG em arquivo temporário usando crate `qrcode` + `image`
- No template Typst, se `prova.qr_gabarito` habilitado (novo campo), incluir no rodapé:
  ```typst
  #align(right)[#image("/tmp/qr_[id].png", width: 2cm)]
  ```
  O texto do QR code é `"Gabarito: [titulo da prova]"` — não requer URL real, apenas texto legível

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN qr_gabarito INTEGER NOT NULL DEFAULT 0;
```

**Backend** (`provas.rs`): incluir `qr_gabarito: bool` em create/update/list/get

**Frontend** (`provas/ProvaEditor.tsx`): checkbox "Incluir QR code do gabarito no rodapé" no painel de exportação
