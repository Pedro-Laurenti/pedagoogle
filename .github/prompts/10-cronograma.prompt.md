---
name: 10-cronograma
description: Melhorias na página de Cronograma
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Refatorar a página `src/app/(main)/cronograma/page.tsx`.

## Mudanças

1. **Restrição de carga horária:** só é possível criar aulas dentro do horário de funcionamento definido nas configurações (`hora_entrada`, `aulas_por_dia`, `minutos_por_aula`). O backend deve rejeitar aulas fora desse intervalo.

2. **Click na grade para criar aula:** clicar em uma célula vazia da grade já deve abrir o modal com dia e horário pré-preenchidos

3. **Recorrência no formulário:** adicionar campo de recorrência usando `InputMultiSelect` com os dias da semana — permite criar a mesma aula em múltiplos dias de uma vez

4. **Remover campos desnecessários:** remover seletores de "semestre" e "bimestre" do formulário (não são mais necessários com a carga horária por configuração)

5. **"Turma"** some do formulário se `usar_turmas = false`

## Backend

- Atualizar validação de horário em `create_aula` e `update_aula` consultando `configuracoes` (`hora_entrada`, `aulas_por_dia`, `minutos_por_aula`)
- Suporte a criação em lote (recorrência): receber lista de `dias_semana` e criar múltiplas aulas

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
