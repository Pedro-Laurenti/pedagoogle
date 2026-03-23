---
name: notas-extra
description: Vinculação nota→turma, nota máxima, boletim PDF e histórico
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Vincular prova a turma (🔴)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL;
```

**Backend** (`provas.rs`): incluir `turma_id: Option<i64>` em `create_prova`, `update_prova`, `list_provas`, `get_prova`

**Frontend** (`provas/ProvaEditor.tsx`):
- Select "Turma" (opções de `list_turmas` + "Todas as turmas") no painel de configuração da prova

**Frontend** (`notas/page.tsx`):
- Ao selecionar aluno, filtrar as provas disponíveis no select de `prova_id`:
  ```ts
  provas.filter(p => !aluno.turma_id || !p.turma_id || p.turma_id === aluno.turma_id)
  ```

**Tipos** (`types/index.ts`): em `Prova`, adicionar `turma_id: number | null;`

## 2. Validação de nota máxima (🟡)

**Backend** (`notas.rs`): em `create_nota` e `update_nota`, se `prova_id` fornecido:
- `SELECT valor_total FROM provas WHERE id = ?1` — retornar `Err("Nota não pode exceder o valor total da prova".into())` se `valor > valor_total`
- Retornar `Err("Nota não pode ser negativa".into())` se `valor < 0.0`

**Frontend** (`notas/page.tsx`):
- Ao selecionar prova no modal, atualizar `maxNota` com `prova.valor_total`
- Input `valor`: `min="0"` e `max={maxNota ?? 10}`

## 3. Exportar boletim PDF por aluno (🟡)

**Backend** (`typst_pdf.rs`):
```rust
pub fn export_boletim_pdf(aluno_id: i64, path: String) -> Result<(), String>
```
- Buscar aluno + notas com JOIN em provas e materias
- Agrupar por matéria; calcular média ponderada por matéria (`Σ(nota × valor_total) / Σ(valor_total)`)
- Ler `nota_minima` das configurações para determinar situação (Aprovado / Em Recuperação / Reprovado)
- Gerar Typst com tabela: Matéria | Provas e Notas | Média | Situação
- Registrar `export_boletim_pdf` em `lib.rs`

**Frontend** (`notas/page.tsx`):
- Botão `MdPictureAsPdf` "Exportar Boletim" visível quando aluno está selecionado
- Usa `dialog.save({ defaultPath: "boletim_[nome].pdf", filters: [{ name: "PDF", extensions: ["pdf"] }] })`

## 4. Histórico de edições de notas (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE notas ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
```

**Backend** (`notas.rs`): incluir `updated_at = datetime('now')` no UPDATE em `update_nota`; retornar `updated_at` na struct `Nota`

**Frontend** (`notas/page.tsx`): coluna "Atualizado" na tabela de notas (formatada como `dd/MM/yyyy HH:mm`)
