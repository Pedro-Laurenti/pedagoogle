---
name: 09-notas
description: Melhorias na página de Notas
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Refatorar a página `src/app/(main)/notas/page.tsx`.

## Filtros

Adicionar filtros em cascata na página principal (nessa ordem):
1. Bimestre
2. Ano letivo
3. Turma (só se `usar_turmas = true`)
4. Matéria
5. Aluno

## Formulário de lançamento

1. **Turma → Aluno (cascata):** se `usar_turmas = true`, selecionar turma primeiro para filtrar a lista de alunos
2. **Categoria integrada com Provas:**
   - Se o usuário selecionar a categoria "Prova": exibir selects em cascata: Matéria → Bimestre → Ano → lista das provas filtradas
   - Categoria "Prova" deve ter uma flag no backend que a vincula ao sistema de provas
3. Remover campo "descrição" (frontend + backend: `ALTER TABLE notas DROP COLUMN descricao` ou migration equivalente)
4. Validar: valor não pode ultrapassar o valor total da matéria no bimestre (buscar na configuração)

## Backend

- Atualizar `list_notas` para suportar filtros: `bimestre`, `ano`, `turma_id`, `materia_id`, `aluno_id`
- Atualizar `create_nota` / `update_nota` removendo parâmetro `descricao`
- Adicionar flag `vincula_provas: bool` na tabela `categoria_lancamentos`

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
