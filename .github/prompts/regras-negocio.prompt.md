---
name: regras-negocio
description: Validações backend e frontend ainda ausentes
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Validações backend

### `notas.rs`
Em `create_nota` e `update_nota`:
- Se `prova_id` fornecido: buscar `valor_total` com `SELECT valor_total FROM provas WHERE id=?1`; retornar `Err("Nota não pode exceder o valor total da prova".into())` se `valor > valor_total`
- Retornar `Err("Nota não pode ser negativa".into())` se `valor < 0.0`

### `materias.rs`
Em `delete_materia`:
- `SELECT COUNT(*) FROM provas WHERE materia_id = ?1` — se > 0, retornar `Err("Existem provas vinculadas a esta matéria. Remova-as antes de excluir.".into())`

### `alunos.rs`
Em `delete_aluno`:
- `SELECT COUNT(*) FROM notas WHERE aluno_id = ?1` — se > 0, retornar `Err("Existem notas vinculadas a este aluno. Remova-as antes de excluir.".into())`

## 2. Erros descritivos no Rust

Em todos os módulos, criar função auxiliar local e usar em vez de `.map_err(|e| e.to_string())`:
```rust
fn map_db_err(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("UNIQUE constraint failed") {
        return "Já existe um registro com esse valor.".into();
    }
    if s.contains("FOREIGN KEY constraint failed") {
        return "Não é possível excluir: existem registros vinculados.".into();
    }
    s
}
```
Aplicar em `create_*`, `update_*`, `delete_*` em: `alunos.rs`, `materias.rs`, `provas.rs`, `notas.rs`, `cronograma.rs`, `turmas.rs`

## 3. Validações frontend

### `provas/ProvaEditor.tsx`
- **Múltipla escolha**: ao salvar, verificar se cada questão do tipo `multipla_escolha` tem pelo menos uma opção com `correta: true`; se não, chamar `notify("Questão X: marque ao menos uma opção correta", "error")` e bloquear submit
- **Verdadeiro/Falso**: ao salvar, verificar se cada item de questão V/F tem resposta definida (`"V"` ou `"F"`); se algum estiver vazio, bloquear
- **Data passada**: se `prova.data` < `new Date().toISOString().slice(0, 10)`, chamar `window.confirm("A data da prova está no passado. Deseja salvar assim?")` antes de prosseguir

### `notas/page.tsx`
- Input `valor` no modal: `min="0"`, `max` atualizado dinamicamente ao selecionar prova (`provas.find(p => p.id === form.prova_id)?.valor_total ?? 10`)
- Validar no `handleSave` antes de invocar backend: `if (form.valor < 0 || (maxNota !== null && form.valor > maxNota)) return notify("Valor inválido", "error")`
