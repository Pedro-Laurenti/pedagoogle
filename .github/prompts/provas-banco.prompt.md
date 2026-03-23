---
name: provas-banco
description: Banco de questões reutilizável, embaralhamento e metadados de questão
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Banco de questões (🔴)

**Backend** (`db.rs` → `migrate()`):
```sql
CREATE TABLE IF NOT EXISTS banco_questoes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo        TEXT NOT NULL DEFAULT 'dissertativa',
    enunciado   TEXT NOT NULL DEFAULT '',
    opcoes      TEXT NOT NULL DEFAULT '[]',
    valor       REAL NOT NULL DEFAULT 1.0,
    tags        TEXT NOT NULL DEFAULT '',
    dificuldade TEXT NOT NULL DEFAULT 'médio'
);
```

**Backend** (`provas.rs`) — novos comandos:
- `list_banco_questoes() -> Vec<BancoQuestao>`
- `create_banco_questao(tipo, enunciado, opcoes, valor, tags, dificuldade) -> i64`
- `update_banco_questao(id, tipo, enunciado, opcoes, valor, tags, dificuldade) -> ()`
- `delete_banco_questao(id) -> ()`
- `import_from_banco(banco_id: i64, prova_id: i64) -> ()` — copia campos para `questoes` com `prova_id` e `ordem = MAX(ordem)+1`

**Modelo** (`models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct BancoQuestao {
    pub id: i64, pub tipo: String, pub enunciado: String,
    pub opcoes: String, pub valor: f64, pub tags: String, pub dificuldade: String,
}
```

**Frontend** (`provas/page.tsx`):
- Adicionar aba "Banco de Questões" (toggle ou tabs) ao lado da lista de provas
- CRUD de questões do banco via modal (campos: tipo select, enunciado, valor, tags, dificuldade select fácil/médio/difícil)
- Filtro textual client-side por enunciado/tags

**Frontend** (`provas/ProvaEditor.tsx`):
- Botão `MdLibraryAdd` "Importar do Banco" no painel de questões
- Abre modal com lista de `list_banco_questoes`; clicar em questão chama `import_from_banco` e recarrega questões da prova

**Tipos** (`types/index.ts`):
```ts
export interface BancoQuestao { id: number; tipo: string; enunciado: string; opcoes: string; valor: number; tags: string; dificuldade: string; }
```

## 2. Tags e dificuldade nas questões comuns (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE questoes ADD COLUMN tags TEXT NOT NULL DEFAULT '';
ALTER TABLE questoes ADD COLUMN dificuldade TEXT NOT NULL DEFAULT 'médio';
```

**Backend** (`provas.rs`): incluir `tags` e `dificuldade` em `replace_questoes` (já recebe `QuestaoInput`) e no SELECT em `list_questoes`

**Frontend** (`provas/ProvaEditor.tsx`): exibir inputs "Tags" e select "Dificuldade" no editor de cada questão (colapsados por padrão)

## 3. Embaralhar questões/opções (🟡)

**Backend** (`provas.rs`):
```rust
pub fn export_prova_pdf_embaralhada(id: i64, path: String, versao: String) -> Result<(), String>
```
- Carregar questões da prova; embaralhar ordem com `sort_by_key(|_| rand::random::<u32>())` (adicionar crate `rand` no `Cargo.toml`)
- Para questões de múltipla escolha (`tipo == "multipla_escolha"`): deserializar `opcoes`, embaralhar, reserializar
- Prefixar título com `"Versão {versao} — "` antes de passar ao template Typst

**Frontend** (`provas/ProvaEditor.tsx`):
- Botão `MdShuffle` "Exportar Versão" ao lado do botão "Exportar PDF"
- Abre dialog com campo "Versão" (ex: A, B, C) e `dialog.save()` para path
