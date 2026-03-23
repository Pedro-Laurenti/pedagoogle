---
name: 02-componente-modal
description: Cria o componente Modal padronizado do sistema
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Criar o componente `src/components/Modal.tsx` que substituirá TODOS os `<div className="modal modal-open">` hardcoded do projeto.

## Especificação do componente

```tsx
interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  // Tipo do modal
  variant?: "form" | "confirm" | "info";
  // Cores DaisyUI: "primary" | "error" | "warning" | "success" | "info"
  color?: "primary" | "error" | "warning" | "success" | "info";
  // Para variant="confirm"
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm?: () => void;
  // Para variant="form" e "info"
  children?: React.ReactNode;
  // Largura máxima (DaisyUI: "sm" | "md" | "lg" | "xl")
  size?: "sm" | "md" | "lg" | "xl";
}
```

## Comportamentos por variant

### `variant="confirm"` (padrão: confirmação de exclusão)
- Exibe `title` + mensagem via `children`
- Botão cancelar (`cancelLabel`, padrão: "Cancelar")
- Botão confirmar (`confirmLabel`, padrão: "Confirmar") com a cor definida em `color`
- Cor padrão: `error` (para confirmação de exclusão)
- Deve chamar `onConfirm()` ao confirmar e `onClose()` ao cancelar

### `variant="info"`
- Exibe `title` + `children`
- Apenas botão "Fechar"

### `variant="form"`
- Exibe `title` + `children` (o formulário completo)
- Sem botões fixos — os botões ficam dentro do `children`
- Suporta scroll interno se o conteúdo for longo

## Regras

- Fechar ao clicar no backdrop (`modal-backdrop`)
- Usar `modal modal-open` do DaisyUI
- `size` mapeia para `modal-box max-w-sm/md/lg/xl`
- O botão de confirmação de exclusão deve SEMPRE ter `color="error"`
- Usar somente `react-icons/md` para ícones (ex: `MdClose`, `MdWarning`)
