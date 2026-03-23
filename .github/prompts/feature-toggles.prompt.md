---
name: feature-toggles
description: Configurações de ativação/desativação de módulos — suporte a homeschooling e escolas simples
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Permitir que o sistema seja usado tanto em escolas tradicionais quanto em contextos mais simples (homeschooling, professor autônomo, escola pequena), desativando módulos desnecessários via checkboxes nas configurações. Itens desativados desaparecem da barra lateral e seus campos relacionados ficam ocultos nos formulários.

---

## 1. Backend — novas colunas em `configuracoes`

**`db.rs` → `migrate()`** — adicionar ao final:
```sql
ALTER TABLE configuracoes ADD COLUMN usar_turmas       INTEGER NOT NULL DEFAULT 1;
ALTER TABLE configuracoes ADD COLUMN usar_professores  INTEGER NOT NULL DEFAULT 1;
ALTER TABLE configuracoes ADD COLUMN usar_frequencia   INTEGER NOT NULL DEFAULT 1;
ALTER TABLE configuracoes ADD COLUMN usar_recuperacao  INTEGER NOT NULL DEFAULT 1;
```

**`models.rs`** — adicionar campos na struct `Configuracoes`:
```rust
pub usar_turmas: bool,
pub usar_professores: bool,
pub usar_frequencia: bool,
pub usar_recuperacao: bool,
```

**`configuracoes.rs`** — incluir os quatro campos em `get_configuracoes` (leitura via `r.get(N)?`) e em `save_configuracoes` (parâmetro + UPDATE SQL). Mapear `INTEGER` → `bool` com `r.get::<_, i64>(N)? != 0` na leitura e `config.usar_turmas as i64` na escrita.

---

## 2. Frontend — tipos

**`src/types/index.ts`** — acrescentar na interface `Configuracoes` (sem remover campos existentes):
```ts
usar_turmas: boolean;
usar_professores: boolean;
usar_frequencia: boolean;
usar_recuperacao: boolean;
```

---

## 3. Frontend — página de configurações

**`configuracoes/page.tsx`** — adicionar seção "Módulos ativos" com três checkboxes usando o padrão DaisyUI:

```tsx
<fieldset className="fieldset">
  <legend className="fieldset-legend">Módulos ativos</legend>
  <p className="text-sm text-base-content/60 mb-3">
    Desative módulos que não se aplicam ao seu contexto (ex.: homeschooling).
  </p>
  <label className="flex items-center gap-2 cursor-pointer mb-2">
    <input type="checkbox" className="checkbox" checked={form.usar_turmas}
      onChange={(e) => setForm({ ...form, usar_turmas: e.target.checked })} />
    Turmas
  </label>
  <label className="flex items-center gap-2 cursor-pointer mb-2">
    <input type="checkbox" className="checkbox" checked={form.usar_professores}
      onChange={(e) => setForm({ ...form, usar_professores: e.target.checked })} />
    Professores
  </label>
  <label className="flex items-center gap-2 cursor-pointer mb-2">
    <input type="checkbox" className="checkbox" checked={form.usar_frequencia}
      onChange={(e) => setForm({ ...form, usar_frequencia: e.target.checked })} />
    Frequência (chamada)
  </label>
  <label className="flex items-center gap-2 cursor-pointer">
    <input type="checkbox" className="checkbox" checked={form.usar_recuperacao}
      onChange={(e) => setForm({ ...form, usar_recuperacao: e.target.checked })} />
    Recuperação
  </label>
</fieldset>
```

Garantir que os valores sejam incluídos no payload do `save_configuracoes`.

---

## 4. Frontend — navegação

**`src/lib/navigation.ts`** — adicionar campo opcional `feature` na interface `MenuItem`:
```ts
export interface MenuItem {
  // ...campos existentes...
  feature?: "usar_turmas" | "usar_professores" | "usar_frequencia" | "usar_recuperacao";
}
```

Nos itens correspondentes do array `menuItems`, adicionar o campo:
| Rota | `feature` |
|---|---|
| `/turmas` | `"usar_turmas"` |
| `/professores` | `"usar_professores"` |
| `/frequencia` | `"usar_frequencia"` |
| `/recuperacao` | `"usar_recuperacao"` |

---

## 5. Frontend — Sidebar filtra por feature

**`src/components/Sidebar.tsx`**:

1. Importar `invokeCmd` de `@/utils/tauri` e o tipo `Configuracoes` de `@/types`
2. Adicionar estado e carga das configs:
```tsx
const [config, setConfig] = useState<Configuracoes | null>(null);
useEffect(() => {
  invokeCmd<Configuracoes>("get_configuracoes").then(setConfig).catch(() => {});
}, []);
```
3. Ao filtrar `sidebarItems`, acrescentar verificação de feature:
```tsx
const sidebarItems = menuItems.filter((item) => {
  if (item.showInSidebar === false) return false;
  if (item.feature && config && !config[item.feature]) return false;
  return true;
});
```
> Enquanto `config` ainda é `null` (carregando), todos os itens são mostrados para evitar flash de conteúdo ausente.

---

## 6. Frontend — ocultar campos vinculados

Quando um módulo estiver desativado, ocultar campos relacionados nos formulários:

- **`usar_turmas = false`**: ocultar o campo "Turma" em `alunos/page.tsx`
- **`usar_professores = false`**: ocultar o campo "Professor" em `materias/page.tsx`
- **`usar_frequencia = false`**: ocultar o link/seção de frequência onde for referenciado
- **`usar_recuperacao = false`**: ocultar o link/seção de recuperação onde for referenciado

Para isso, cada página afetada deve carregar as configs com `invokeCmd<Configuracoes>("get_configuracoes")` no `useEffect` (junto ao `load` existente) e condicionar a renderização do campo com `{config?.usar_turmas && <fieldset>...</fieldset>}`.
