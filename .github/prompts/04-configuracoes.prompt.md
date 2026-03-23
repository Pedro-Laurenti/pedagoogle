---
name: 04-configuracoes
description: Melhorias na página de Configurações
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Corrigir bugs e adicionar funcionalidades na página `src/app/(main)/configuracoes/page.tsx`.

## Correções e melhorias

### 1. Bug de scroll duplo
- A sidebar deve ser sempre `fixed`
- O conteúdo principal deve ter `overflow-y: auto` apenas no container interno
- Nunca deve existir dois scrollbars simultâneos na tela

### 2. Carga horária (nova seção no backend e frontend)
Adicionar nova seção "Carga Horária" nas configurações. Backend: adicionar colunas à tabela `configuracoes` (migration nova):

| Campo | Tipo | Padrão |
|---|---|---|
| `aulas_por_dia` | INTEGER | 6 |
| `minutos_por_aula` | INTEGER | 45 |
| `hora_entrada` | TEXT | "07:00" |
| `dias_letivos_semana` | INTEGER | 5 |

Frontend exibe (calculados, somente leitura):
- Horário de encerramento (entrada + aulas × minutos)
- Horas/aulas por semana, mês, bimestre, semestre e ano

### 3. Auto-update (nova seção)
Adicionar botão "Verificar atualizações" que chama um comando Rust `check_update` que:
- Faz GET em `https://github.com/Pedro-Laurenti/pedagoogle/releases/latest` (via `reqwest`)
- Compara a tag com a versão atual (`tauri::VERSION` ou `env!("CARGO_PKG_VERSION")`)
- Se houver versão nova: exibe modal informativo com link para download
- Migrações de banco são tratadas automaticamente pelo sistema de migrations existente

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
