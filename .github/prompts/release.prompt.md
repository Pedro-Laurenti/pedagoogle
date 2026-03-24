---
description: Publica uma nova versão do Pedagoogle de forma correta, atualizando apenas os arquivos necessários e evitando corrupção do Cargo.lock.
---

# Release de nova versão do Pedagoogle

Siga EXATAMENTE os passos abaixo. Não use "Localizar e substituir em todos os arquivos" — isso corrompe o `Cargo.lock`.

## Arquivos que devem ter a versão atualizada (e apenas estes)

| Arquivo | Campo |
|---|---|
| `src/package.json` | `"version"` |
| `src-tauri/tauri.conf.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` na seção `[package]` (linha 3) |

O `Cargo.lock` **nunca deve ser editado manualmente**. Ele é regenerado automaticamente via `cargo update`.

## Passo a passo

### 1. Determine a nova versão

Consulte [release.instructions.md](../instructions/release.instructions.md) para escolher o bump correto (patch / minor / major).

### 2. Atualize a versão nos três arquivos

Edite **somente a linha `"version"`** em cada arquivo abaixo:

**`src/package.json`** — linha `"version"`:
```json
"version": "X.Y.Z",
```

**`src-tauri/tauri.conf.json`** — linha `"version"`:
```json
"version": "X.Y.Z",
```

**`src-tauri/Cargo.toml`** — seção `[package]`, campo `version`:
```toml
version = "X.Y.Z"
```

### 3. Regenere o Cargo.lock

```bash
cd src-tauri
cargo update
```

Isso garante que o lock file fique consistente sem conter versões yanked.

### 4. Commit, tag e push

```bash
cd /home/pedro/DEVP/pedagoogle
git add src/package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "X.Y.Z"
git push origin main

git tag vX.Y.Z
git push origin vX.Y.Z
```

O CI compila para Windows e Linux e cria um rascunho de Release. Acesse **github.com/Pedro-Laurenti/pedagoogle/releases**, revise e clique **Publish release**.

## O que NÃO fazer

- ❌ Ctrl+Shift+F → "Substituir em todos os arquivos" com a versão — isso substitui versões de dependências dentro do `Cargo.lock` e causa erros como `failed to select a version for the requirement`.
- ❌ Editar `Cargo.lock` manualmente.
- ❌ Alterar versões de dependências em `Cargo.toml` (ex: `tauri = "2"`) achando que são a versão do app.
