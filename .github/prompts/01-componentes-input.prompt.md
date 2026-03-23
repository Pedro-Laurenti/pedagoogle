---
name: 01-componentes-input
description: Cria todos os componentes de input padronizados do sistema
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Criar todos os componentes de input padronizados em `src/components/inputs/`, que serão usados em TODO o sistema. Nenhum input hardcoded é permitido após essa etapa.

## Componentes a criar

Todos os componentes devem seguir o estilo DaisyUI, usando `fieldset` + `legend` conforme o padrão do projeto.

### 1. `InputTelefone.tsx`
- Máscara brasileira `(00) 00000-0000`
- Validação: erro se não estiver preenchido completamente
- Props: `value`, `onChange`, `label?`, `required?`, `error?`

### 2. `InputEmail.tsx`
- Validação de formato de e-mail
- Props: `value`, `onChange`, `label?`, `required?`, `error?`

### 3. `InputCPF.tsx`
- Máscara brasileira `000.000.000-00`
- Validação: erro se não estiver preenchido completamente
- Props: `value`, `onChange`, `label?`, `required?`, `error?`

### 4. `InputMultiSelect.tsx`
- Seleção múltipla com tags (badges) acumulativas
- Cada tag tem botão `×` para remoção individual
- Props: `options: {value: string|number, label: string}[]`, `value: (string|number)[]`, `onChange`, `label?`, `placeholder?`

### 5. `InputData.tsx`
- Picker de datas em português brasileiro
- Formato de exibição: `dd/mm/aaaa`
- Props: `value`, `onChange`, `label?`, `required?`, `min?`, `max?`

### 6. `InputHora.tsx`
- Picker de horas nativo (`type="time"`) estilizado com DaisyUI
- Props: `value`, `onChange`, `label?`, `required?`

### 7. `InputImagem.tsx`
- Usa `@tauri-apps/plugin-dialog` para abrir seletor de arquivo
- Copia o arquivo para o `appDataDir` e retorna o caminho absoluto
- Exibe preview da imagem selecionada
- Botão "Remover" para limpar
- Props: `value: string`, `onChange: (path: string) => void`, `label?`

### 8. `InputTexto.tsx`
- Input de texto simples estilizado DaisyUI
- Props: `value`, `onChange`, `label?`, `placeholder?`, `required?`, `error?`, `type?`

### 9. `InputTextArea.tsx`
- Textarea estilizada DaisyUI
- Props: `value`, `onChange`, `label?`, `placeholder?`, `required?`, `rows?`

### 10. `InputCheckbox.tsx`
- Checkbox estilizada DaisyUI
- Props: `checked`, `onChange`, `label`, `disabled?`

### 11. `InputRadio.tsx`
- Grupo de radio buttons estilizado DaisyUI
- Props: `options: {value: string, label: string}[]`, `value`, `onChange`, `label?`

### 12. `RichEditor` (já existe em `src/components/RichEditor.tsx`)
- Padronizar com o mesmo estilo `fieldset` + `legend` dos demais
- Aceitar prop `label?`

## Exportação

Criar `src/components/inputs/index.ts` exportando todos os componentes acima.

## Regras

- Nenhum componente deve usar `<input>` ou `<textarea>` hardcoded fora de `src/components/inputs/`
- Todos os erros de validação exibem borda vermelha + mensagem abaixo do campo
- Usar somente `react-icons/md` para ícones internos
