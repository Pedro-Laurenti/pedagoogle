---
name: 06-materias
description: Melhorias na página de Matérias
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Refatorar a página `src/app/(main)/materias/page.tsx`.

## Mudanças

### Frontend

1. Quando `usar_turmas = false`: remover filtro por turma
2. Remover campo "descrição" do formulário (frontend + backend)
3. Modal de criação/edição dividido em **steps** para evitar scroll:
   - **Step 1** — Informações básicas: nome, ícone, cor
   - **Step 2** — Alocação: professores (se `usar_professores = true`), turmas (se `usar_turmas = true`)
4. Campo turma some se `usar_turmas = false`
5. Campo professor some se `usar_professores = false`

### Campo "Aulas/Semana" → painel de carga horária

Substituir o input simples de "Aulas/Semana" por um painel informativo + configurável:

- Input: quantas aulas por semana esta matéria tem (respeita `aulas_por_dia` das configurações como limite)
- Calculados e exibidos (somente leitura):
  - Horários livres por dia, por semana
  - Aulas por mês, bimestre, semestre, ano

### Backend
- Criar migration removendo coluna `descricao` (SQLite não suporta `DROP COLUMN` antes do 3.35 — recriar tabela se necessário, ou simplesmente parar de usar o campo)

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
