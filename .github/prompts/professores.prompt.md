---
name: professores
description: Cadastro de professores e associação com matérias
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Cadastro de professores (🟡)

### Backend (`db.rs` → `migrate()`)
```sql
CREATE TABLE IF NOT EXISTS professores (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    nome  TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT ''
);
ALTER TABLE materias ADD COLUMN professor_id INTEGER REFERENCES professores(id) ON DELETE SET NULL;
```

### Backend — criar `src-tauri/src/professores.rs`
Comandos:
- `list_professores() -> Vec<Professor>`
- `create_professor(nome: String, email: String) -> Result<i64, String>`
- `update_professor(id: i64, nome: String, email: String) -> Result<(), String>`
- `delete_professor(id: i64) -> Result<(), String>` — verificar se existem matérias vinculadas antes de excluir

Registrar em `lib.rs` e declarar `mod professores;`

### Backend (`materias.rs`)
- `create_materia` / `update_materia`: substituir parâmetro `professor: String` por `professor_id: Option<i64>`
- `list_materias`: LEFT JOIN professores para retornar `professor_nome: Option<String>`
- Remover coluna `professor` do INSERT/UPDATE (manter na tabela com DEFAULT '' para não quebrar dados existentes)

### Frontend — criar `src/app/(main)/professores/page.tsx`
CRUD padrão com modal. Campos: nome, email. Adicionar entrada "Professores" no menu em `navigation.ts` (ícone: `MdPersonOutline`)

### Frontend (`materias/page.tsx`)
- Substituir input texto "Professor" por select `professor_id` (opções de `list_professores` + "Nenhum")
- Exibir `professor_nome` na tabela

### Tipos (`types/index.ts`)
```ts
export interface Professor { id: number; nome: string; email: string; }
// Materia: substituir professor: string por professor_id: number | null; professor_nome?: string;
```
