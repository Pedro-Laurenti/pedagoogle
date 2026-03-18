# Pedagoogle

Plataforma de edicao e gestao escolar. Desktop standalone, totalmente offline.

## Stack

- **Frontend**: Next.js (JavaScript) + TailwindCSS + DaisyUI
- **Backend**: Tauri 2 (Rust)
- **Banco de dados**: SQLite (local, `pedagoogle.db`)

## Estrutura

```
pedagoogle/
├── src/               # Frontend Next.js
│   ├── app/           # Paginas (App Router)
│   ├── components/    # Componentes reutilizaveis
│   └── utils/         # Utilitarios (Tauri invoke)
└── src-tauri/         # Backend Rust
    └── src/
        ├── db.rs       # Conexao e migrations
        ├── models.rs   # Structs de dados
        ├── materias.rs # Comandos de materias
        ├── alunos.rs   # Comandos de alunos
        ├── provas.rs   # Comandos de provas e questoes
        ├── notas.rs    # Comandos de notas
        ├── cronograma.rs # Comandos de aulas
        ├── pdf.rs      # Geracao de PDF
        └── lib.rs      # Entry point Tauri
```

## Desenvolvimento

```bash
# Instalar dependencias do frontend
cd src && npm install

# Rodar em modo desenvolvimento (abre janela Tauri)
cd src-tauri && cargo tauri dev

# Build final
cd src-tauri && cargo tauri build
```

## Funcionalidades

- **Dashboard**: resumo geral
- **Materias**: CRUD completo
- **Provas**: criacao com editor de questoes (dissertativa, multipla escolha, V/F, lacunas, associacao, ordenar), exportacao PDF
- **Alunos**: CRUD completo
- **Notas**: lancamento flexivel por aluno (provas, atividades, trabalhos)
- **Cronograma**: grade semanal de aulas
