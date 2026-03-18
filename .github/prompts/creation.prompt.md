---
name: creation
description: Criação inicial
agent: agent
model: Claude Sonnet 4.6 (copilot)
---

RESPEITE SEMPRE [RULES](../instructions/rules.instructions.md)

Quero criar uma plataforma de edição e gestão escolar, que será um app desktop standalone.

Essa plataforma se chamará Pedagoogle.

# Stack obrigatória
- Banco de dados local: SQLite
- Backend: Tauri (Rust)
- Frontend: Next.js (JavaScript)
- Estilização: TailwindCSS + DaisyUI

# Arquitetura
- Separação clara entre frontend e backend
- Backend (Tauri/Rust) responsável por:
  - regras de negócio
  - acesso ao banco SQLite
  - geração de PDF
- Frontend (Next.js) responsável apenas pela interface e interação com usuário
- Comunicação via comandos do Tauri

# Funcionalidades

## 1. Dashboard
- Tela inicial ao abrir o app
- Exibir resumo futuro (placeholder inicialmente)

## 2. Provas
Fluxo:
1. Usuário acessa "Provas" na sidebar
2. Visualiza lista de provas existentes
3. Pode criar nova prova ou editar existente

Funcionalidades:
- Criar, editar, excluir provas
- Configurar:
  - título
  - descrição
  - matéria
  - data
- Editor de questões:
  - questões dissertativas
  - questões de múltipla escolha
  - alternativas
  - verdadeiro ou falso
  - completar lacunas
  - associação (colunas)
  - ordenar/sequenciar
- Pode inserir imagens no enunciado de uma questão
- Estilização da prova:
  - margens
  - cabeçalho
  - rodapé
- Salvamento:
  - salvar no banco (editável)
  - exportar como PDF

## 3. Matérias
- CRUD de matérias
- Campos:
  - nome
  - descrição
- Relacionamento com provas e aulas

## 4. Alunos
- CRUD de alunos
- Campos:
  - nome
- Listagem e edição

## 5. Notas
- Lançamento de notas por aluno
- Calculo deve ser responsivo, ou seja, dependendo do professor pode haver mais lançamentos (não apenas prova), como atividades complementares, trabalhos, simulados e outras provas
- Relacionamento:
  - aluno
  - prova
- Funcionalidades:
  - atribuir nota
  - editar nota
  - visualizar histórico de notas por aluno
  - visualizar notas por prova

## 6. Cronograma de Aulas
- Cadastro de horários de aula
- Campos:
  - matéria
  - dia da semana
  - horário início/fim
- Visualização em formato de grade semanal

# Requisitos técnicos
- Uso de SQLite como arquivo local (ex: pedagoogle.db)
- Estrutura de tabelas normalizada
- Uso de migrations
- Backend deve expor comandos organizados por domínio:
  - provas
  - alunos
  - matérias
  - notas
  - cronograma
- Código modular

# Extras desejáveis
- Geração de PDF das provas
- Interface simples e objetiva
- Navegação por sidebar
- Feedback visual de ações (sucesso/erro)

# Objetivo
Criar um sistema completo de gestão de provas e apoio escolar, totalmente offline, leve e funcional.