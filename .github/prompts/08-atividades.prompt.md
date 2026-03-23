---
name: 08-atividades
description: Criar nova página de Atividades (baseada em Provas)
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Criar a página de Atividades (`src/app/(main)/atividades/`), que é uma variação da página de Provas com as diferenças listadas abaixo.

## Base

Copiar toda a estrutura de Provas (backend + frontend), incluindo:
- Editor de questões com todos os tipos
- Bloco de texto livre
- Espaço de rascunho por questão
- Cabeçalho do PDF no mesmo estilo
- Fluxo em steps/abas

## Diferenças em relação a Provas

### Título
Não gerado automaticamente. Estrutura fixa no PDF:
```
[NOME DA MATÉRIA]           ← fonte pequena
[TÍTULO DA ATIVIDADE]       ← título definido pelo usuário, fonte maior
```

### Campo "Vale nota?"
- Checkbox opcional no formulário
- Se marcado: exibir campos de Valor e Nota no cabeçalho do PDF
- Se desmarcado: ocultar campos de valor/nota

### Identificação no sistema
- Tabela separada `atividades` no banco (não misturar com `provas`)
- Registrar em `navigation.ts` com rota `/atividades`
- Adicionar ao grupo "Conteúdo" na sidebar

## Backend

Criar `src-tauri/src/atividades.rs` seguindo o mesmo padrão de `provas.rs`:
- `list_atividades`, `create_atividade`, `update_atividade`, `delete_atividade`
- Registrar todos os comandos em `lib.rs`
- Adicionar migration para a nova tabela

## Regras
- Todos os inputs devem ser padronizados usando os componentes de input (`src/components/inputs/`)
- Os modais de edição e criação devem utilizar o componente `Modal` padronizado
- Deve ter um modal de confirmação de exclusão utilizando o componente `Modal` padronizado
