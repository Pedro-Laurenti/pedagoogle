---
description: Processo obrigatório para publicar uma nova versão do Pedagoogle.
applyTo: "**/Cargo.toml,**/tauri.conf.json,**/package.json"
---

# Publicar uma Nova Versão

## Como funcionar

A versão é controlada **pela tag Git**. O GitHub Actions atualiza automaticamente `Cargo.toml`, `tauri.conf.json` e `package.json` durante o build com base na tag — não é necessário editar esses arquivos manualmente.

## Passo a passo

```bash
# 1. Commit todas as mudanças de código
git add .
git commit -m "1.2.2"
git push origin main

# 2. Crie a tag e dispare o CI
git tag v1.2.2
git push origin v1.2.2
```

O CI compila para Windows e Linux e cria um rascunho de Release. Abra **github.com/Pedro-Laurenti/pedagoogle/releases**, revise e clique **Publish release**.

## Convenção de versão (SemVer)

| Tipo de mudança | Bump |
|---|---|
| Correção de bug | `X.Y.Z+1` (patch) |
| Nova funcionalidade | `X.Y+1.0` (minor) |
| Quebra de compatibilidade / migração de banco | `X+1.0.0` (major) |
