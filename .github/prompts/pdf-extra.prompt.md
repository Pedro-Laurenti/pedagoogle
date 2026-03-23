---
name: pdf-extra
description: Modo duas colunas, orientação paisagem e tamanho fonte no PDF
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [BACKEND](../instructions/backend.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md)

## 1. Modo duas colunas no PDF (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN duas_colunas INTEGER NOT NULL DEFAULT 0;
```

**Backend** (`provas.rs`): incluir `duas_colunas: bool` (mapeado de `INTEGER` com `r.get::<_, i64>(...) != 0`) em create/update/list/get

**Backend** (`typst_pdf.rs`): no `#set page(...)`, se `prova.duas_colunas`, adicionar `columns: 2`; além disso, quando em duas colunas, não usar `block` de largura total para questões — usar layout padrão de fluxo

**Frontend** (`provas/ProvaEditor.tsx`): checkbox `<input type="checkbox">` "Layout duas colunas (questões objetivas)" no painel de configuração da prova

## 2. Orientação paisagem (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE provas ADD COLUMN paisagem INTEGER NOT NULL DEFAULT 0;
```

**Backend** (`provas.rs`): incluir `paisagem: bool` em create/update/list/get

**Backend** (`typst_pdf.rs`): se `prova.paisagem`, adicionar `flipped: true` no `#set page(paper: "a4", flipped: true, ...)`

**Frontend** (`provas/ProvaEditor.tsx`): checkbox "Orientação paisagem" no painel de configuração da prova

> Nota: `tamanho_fonte` configurável já está coberto em `configuracoes-extra.prompt.md` item 3.
