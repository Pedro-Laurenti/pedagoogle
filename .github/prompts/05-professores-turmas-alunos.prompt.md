---
name: 05-professores-turmas-alunos
description: Melhorias nas páginas de Professores, Turmas e Alunos
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Refatorar as páginas de Professores, Turmas e Alunos conforme especificações abaixo.

---

## Professores (`src/app/(main)/professores/page.tsx`)

1. Remover campos: número de celular, e-mail, observações
2. Renomear "especialidade" → "matérias": MultiSelect com todas as matérias do sistema (componente `InputMultiSelect`)
3. Adicionar filtros respeitando `usar_turmas`, `usar_professores`

---

## Turmas (`src/app/(main)/turmas/page.tsx`)

Backend (migration nova):
- Remover colunas `nome` e `ano_letivo` (criar nova tabela ou migration com rename)
- Adicionar coluna `ano` (TEXT, ex: "6º Ano", "1º EM") e `turma` (TEXT, ex: "A", "B")
- Nome exibido no sistema: concatenar `ano + " - " + turma` (ex: "6º Ano - A")

Frontend:
1. Remover campos "ano letivo" e "nome"
2. Adicionar campo "Ano" (texto livre, ex: "6º Ano") e "Turma" (letra A–Z, select ou texto)
3. Relacionar turma com N matérias (usando `InputMultiSelect`) — só quando `usar_turmas = true`
4. Respeitar configuração de carga horária para alocação de matérias

---

## Alunos (`src/app/(main)/alunos/page.tsx`)

Backend:
- Remover campo `matricula`
- ID único gerado automaticamente (já existe via `AUTOINCREMENT`)

Frontend:
1. Remover campo "matrícula" do formulário e da tabela
2. Remover botão "Importar CSV" e todo o código relacionado (frontend + backend)
3. Campo "turma" só aparece se `usar_turmas = true`
4. Quando `usar_turmas = false`: exibir campo obrigatório de matérias do aluno (`InputMultiSelect`)
5. Filtro de turmas desaparece quando `usar_turmas = false`
6. Input de imagem usando componente `InputImagem` padronizado

---

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
