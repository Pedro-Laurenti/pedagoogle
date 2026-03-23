---
name: 07-provas
description: Refatoração completa da página de Provas
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Refatorar completamente a página de Provas (`src/app/(main)/provas/`).

## Remoções (backend + frontend)

- Remover: opção "é prova de recuperação"
- Remover: opção "layout duas colunas"
- Remover: opção "orientação paisagem"
- Remover: botão "exportar versão"
- Remover: banco de questões
- Remover: tags em cada questão
- Remover: cabeçalho personalizado
- Remover: campo "data" (substituído por bimestre/ano)

## Título automático

O título da prova deve ser gerado automaticamente, sem campo editável:
```
AVALIAÇÃO DE [NOME DA MATÉRIA] - [Nº DO BIMESTRE]º BIMESTRE
```

## Cabeçalho do PDF

Implementar o cabeçalho conforme design especificado (duas colunas: logo 2/10 + info 8/10):
- Logo da escola (da configuração `logo_path`)
- Nome da escola, cidade
- Campos: Data (com cidade da config + dia/mês preenchidos pelo aluno), Professor(a), Ano, Turma, Turno, Valor, Nota
- Faixas decorativas na cor primária do tema

## Fluxo em steps/abas

**Aba 1 — Configurações da prova:**
- Matéria (select, obrigatório)
- Bimestre (select: 1º–4º, obrigatório)
- Ano letivo (texto, obrigatório)
- Turma (select, só se `usar_turmas = true`)
- Valor total da prova (número, obrigatório)
- Instruções/Descrição (RichEditor, salvo como HTML no backend)

**Aba 2 — Questões:**
- Botão "Adicionar Questão" (tipos existentes)
- Botão "Adicionar Texto" (bloco de texto livre sem ser questão)
- Cada questão pode ter configuração de espaço em branco de rascunho (em "linhas em branco" — espaçamento de linha, sem traços) 
- Reordenação por drag ou setas

## Filtros na página principal

- Substituir emojis por ícones `react-icons/md`
- Filtros usando componentes de input padronizados

## Validação obrigatória

Campos obrigatórios (toast + borda vermelha se não preenchidos): Matéria, Bimestre, Valor total

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
