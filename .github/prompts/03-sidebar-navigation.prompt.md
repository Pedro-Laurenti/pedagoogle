---
name: 03-sidebar-navigation
description: Agrupa páginas em grupos colapsáveis na sidebar
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Agrupar todos os itens da sidebar (`src/components/Sidebar.tsx` e `src/lib/navigation.ts`) em grupos colapsáveis.

## Grupos sugeridos

| Grupo | Páginas |
|---|---|
| **Escola** | Dashboard, Configurações |
| **Pessoas** | Alunos, Professores, Turmas |
| **Conteúdo** | Matérias, Provas, Atividades |
| **Avaliação** | Notas, Cronograma |

## Requisitos

1. Adicionar campo `group?: string` na interface `MenuItem` em `navigation.ts`
2. Sidebar agrupa visualmente os itens por grupo com um label de seção colapsável
3. O estado colapsado/expandido de cada grupo deve ser persistido no `localStorage`
4. Grupos sem nenhum item visível (filtrados por `feature`) devem ser omitidos
5. A sidebar deve continuar sendo `fixed` e com scroll interno próprio — **nunca** gerar dois scrollbars
6. Usar somente `react-icons/md` para ícones de chevron (`MdExpandMore`, `MdChevronRight`)

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
