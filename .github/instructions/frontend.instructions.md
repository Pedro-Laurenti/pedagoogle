---
description: Requisitos para implementar uma nova página no frontend.
applyTo: "src/app/**/*.tsx,src/components/**/*.tsx"
---

# Nova Página Frontend

## Localização
`src/app/(main)/[rota]/page.tsx` → registrar em `src/lib/navigation.ts` (array `menuItems`)

## Imports padrão
```tsx
"use client";
import { useState, useEffect, useCallback } from "react";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { MinhaEntidade, ToastState } from "@/types";
```

## Estado padrão (CRUD)
```tsx
const [items, setItems]     = useState<T[]>([]);
const [form, setForm]       = useState<Form>(EMPTY);
const [editing, setEditing] = useState<number | null>(null);
const [modal, setModal]     = useState(false);
const [toast, setToast]     = useState<ToastState | null>(null);

const load = useCallback(async () => {
  setItems(await invokeCmd<T[]>("list_..."));
}, []);
useEffect(() => { load(); }, [load]);

function notify(msg: string, type: ToastState["type"] = "success") {
  setToast({ message: msg, type });
  setTimeout(() => setToast(null), 3000);
}
```

## Componentes disponíveis
| Componente | Uso |
|--|--|
| `<Toast message type onClose>` | Notificações — sempre no fim do JSX |
| `<RichEditor value onChange>` | Editor rich text / LaTeX — apenas quando necessário |
| `PageHeader` | **Não incluir** — automático via `(main)/layout.tsx` |

## DaisyUI — padrões obrigatórios
```tsx
// Input
<fieldset className="fieldset">
  <legend className="fieldset-legend">Label</legend>
  <input className="input w-full" />
</fieldset>

// Tabela
<table className="table table-zebra w-full">

// Modal
<div className="modal modal-open">
  <div className="modal-box">
    <div className="modal-action">
      <button className="btn" onClick={() => setModal(false)}>Cancelar</button>
      <button className="btn btn-primary" type="submit">Salvar</button>
    </div>
  </div>
</div>

// Botões na tabela
<button className="btn btn-sm btn-ghost"><MdEdit /></button>
<button className="btn btn-sm btn-ghost text-error"><MdDelete /></button>
```

## Ícones
Somente `react-icons/md` (ex: `MdAdd`, `MdEdit`, `MdDelete`, `MdCopyAll`)

## Tipos novos
Adicionar em `src/types/index.ts` — sem duplicar declarações existentes
