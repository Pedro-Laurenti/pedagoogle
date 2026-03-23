---
name: 11-dashboard-frequencia-limpeza
description: Melhoria do Dashboard, remoção da Frequência e limpeza geral
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Melhorar o Dashboard, remover a página de Frequência e fazer limpeza de código legado.

---

## Dashboard (`src/app/(main)/dashboard/page.tsx`)

Tornar a página mais visual e útil:

1. **Shortcuts rápidos:** cards clicáveis para as páginas principais (Alunos, Matérias, Provas, Notas, Cronograma) com ícone grande e contagem de registros
2. **Gráfico de notas:** gráfico de barras simples (usar `recharts` ou similar já instalado) com médias por matéria
3. **Próximas provas:** lista das próximas provas ordenadas por data
4. **Aulas de hoje:** já existe, manter e melhorar visual
5. Usar somente `react-icons/md` para ícones — sem emojis

---

## Frequência — Remoção completa

1. Remover `src/app/(main)/frequencia/page.tsx`
2. Remover entrada de `frequencia` em `src/lib/navigation.ts`
3. Remover `src-tauri/src/frequencia.rs`
4. Remover registro de comandos de frequência em `lib.rs`
5. Remover tabelas de frequência das migrations (não deletar dados, apenas parar de expor)

---

## Limpeza geral

- Garantir que nenhum emoji (`\u{1F...}`) exista em nenhum arquivo `.tsx` — usar `react-icons/md`
- Remover imports não utilizados após as refatorações anteriores

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
