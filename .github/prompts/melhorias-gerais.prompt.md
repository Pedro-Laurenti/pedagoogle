---
name: melhorias-gerais
description: Corrige todos os bugs e melhorias listados em MELHORIAS.md
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Corrigir todos os bugs e implementar todas as melhorias listadas abaixo. Siga cada seção na ordem.

---

## 1. Professores — campos e relacionamentos

O cadastro de professores está muito vazio. Expanda para incluir:

**Backend (`db.rs` → `migrate()`)** — adicionar colunas e tabela de vínculo:
```sql
ALTER TABLE professores ADD COLUMN telefone    TEXT NOT NULL DEFAULT '';
ALTER TABLE professores ADD COLUMN especialidade TEXT NOT NULL DEFAULT '';
ALTER TABLE professores ADD COLUMN aulas_por_semana INTEGER NOT NULL DEFAULT 0;
ALTER TABLE professores ADD COLUMN observacoes TEXT NOT NULL DEFAULT '';

-- vínculo professor ↔ matéria (muitos para muitos)
CREATE TABLE IF NOT EXISTS professor_materias (
    professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE,
    materia_id   INTEGER NOT NULL REFERENCES materias(id)   ON DELETE CASCADE,
    PRIMARY KEY (professor_id, materia_id)
);

-- vínculo professor ↔ turma (muitos para muitos, opcional)
CREATE TABLE IF NOT EXISTS professor_turmas (
    professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE,
    turma_id     INTEGER NOT NULL REFERENCES turmas(id)      ON DELETE CASCADE,
    PRIMARY KEY (professor_id, turma_id)
);

-- cronograma pessoal do professor (igual à tabela de eventos de cronograma)
CREATE TABLE IF NOT EXISTS professor_cronograma (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    professor_id INTEGER NOT NULL REFERENCES professores(id) ON DELETE CASCADE,
    titulo       TEXT    NOT NULL,
    dia_semana   INTEGER NOT NULL, -- 0=Dom … 6=Sáb
    hora_inicio  TEXT    NOT NULL, -- "HH:MM"
    hora_fim     TEXT    NOT NULL,
    cor          TEXT    NOT NULL DEFAULT '#3b82f6',
    recorrente   INTEGER NOT NULL DEFAULT 1
);
```

**Backend (`professores.rs`)** — expandir comandos:
- `create_professor` e `update_professor`: incluir campos `telefone`, `especialidade`, `aulas_por_semana`, `observacoes`
- Criar `set_professor_materias(professor_id: i64, materia_ids: Vec<i64>) -> Result<(), String>` — DELETE + INSERT em professor_materias
- Criar `set_professor_turmas(professor_id: i64, turma_ids: Vec<i64>) -> Result<(), String>` — mesma lógica
- Criar `list_professor_materias(professor_id: i64) -> Result<Vec<i64>, String>`
- Criar `list_professor_turmas(professor_id: i64) -> Result<Vec<i64>, String>`
- Criar `list_professor_cronograma(professor_id: i64) -> Result<Vec<ProfessorCronograma>, String>`
- Criar `save_professor_cronograma(professor_id: i64, eventos: Vec<ProfessorCronogramaInput>) -> Result<(), String>` — DELETE + INSERT

**`models.rs`** — atualizar struct `Professor` com os novos campos e adicionar `ProfessorCronograma`.

**Frontend (`professores/page.tsx`)** — modal de cadastro com:
- Campos: nome, email, telefone, especialidade, aulas_por_semana, observacoes
- Multi-select de matérias (checkboxes ou tags) — visível sempre
- Multi-select de turmas — visível só quando `usar_turmas === true` (buscar do `configuracoes`)
- Aba ou seção "Cronograma" dentro do modal: grade semanal igual à do cronograma geral, só que pessoal do professor (Seg–Sex, faixas de horário configuráveis)
- Na listagem, mostrar as matérias que o professor leciona e o número de aulas por semana

---

## 2. Turmas — corrigir erro ao salvar

Investigar e corrigir o bug "Erro ao salvar." na página de turmas.

Passos:
1. Ler `src-tauri/src/turmas.rs` e verificar se todos os parâmetros do comando `create_turma` / `update_turma` batem exatamente (nome e tipo) com o que o frontend envia via `invokeCmd`.
2. Ler `src/app/(main)/turmas/page.tsx` e conferir o payload do `invokeCmd`.
3. Corrigir qualquer discrepância de nome de campo (camelCase no TS → snake_case no Rust) ou tipo de dado.
4. Testar o fluxo de criação e edição de turma.

---

## 3. Alunos — corrigir erro ao salvar e adicionar foto

**Bug "Erro ao salvar."**: mesma investigação do item 2 — conferir nomes e tipos dos parâmetros entre frontend e backend.

**Foto do aluno**:
- Substituir o campo de texto para caminho de arquivo por um botão "Escolher foto" que abre o seletor de arquivos nativo do sistema operacional (usar `@tauri-apps/plugin-dialog`, função `open`, filtro `image/*`).
- Após o usuário escolher a imagem, copiar o arquivo para `$APPDATA/pedagoogle/fotos/alunos/` usando os comandos de sistema de arquivos do Tauri (`@tauri-apps/plugin-fs`, funções `copyFile` e `mkdir`). Salvar no banco apenas o nome do arquivo (não o caminho completo).
- Exibir a foto no modal e na listagem usando o caminho resolvido via `convertFileSrc` (plugin `tauri`).
- Qualquer outra imagem salva pelo usuário no app deve seguir o mesmo padrão: copiar para `$APPDATA/pedagoogle/` e guardar só o nome do arquivo.
- No backend (`alunos.rs`), o campo `foto` deve ser `TEXT NOT NULL DEFAULT ''`.

---

## 4. Provas — remover campos desnecessários e melhorar UX

**Remover:**
- Campo "Margens" do formulário/editor de provas — apagar do frontend e do backend (se houver coluna, pode manter a coluna no banco mas ignorar no código).
- Toda a funcionalidade de QR Code — remover botão, lógica e imports relacionados.

**Melhorar UX/UI da listagem de provas:**
- Adicionar barra de filtros visível no topo da página com: Matéria (select), Semestre (select), Categoria (todas / provas normais / recuperação).
- Exibir as provas em cards visuais em vez de tabela simples, mostrando: título, matéria (com cor/ícone da matéria se disponível), data, categoria (badge "Recuperação" em destaque), quantidade de questões.
- Adicionar botão de busca por texto (título da prova).
- Ordenação por data (mais recente primeiro por padrão).

---

## 5. Matérias — corrigir erro de argumento e adicionar cor e ícone

**Corrigir erro `missing required key cargaHorariaSemanal`**:
- Ler `src/app/(main)/materias/page.tsx` e `src-tauri/src/materias.rs`.
- O frontend deve enviar `cargaHorariaSemanal` (camelCase) e o Rust deve receber `carga_horaria_semanal` (snake_case) — Tauri converte automaticamente, então verificar se o campo existe no Rust e se o frontend está enviando com o nome correto.
- Corrigir o payload do `invokeCmd` ou o parâmetro do comando Rust para que os nomes batam.

**Adicionar cor e ícone:**
- No backend (`db.rs`): `ALTER TABLE materias ADD COLUMN cor TEXT NOT NULL DEFAULT '#3b82f6';` e `ALTER TABLE materias ADD COLUMN icone TEXT NOT NULL DEFAULT 'MdBook';`
- No backend (`materias.rs`): incluir `cor` e `icone` em create/update/list.
- No frontend (`materias/page.tsx`): adicionar `<ColorPicker>` e `<IconPicker>` no modal de cadastro, seguindo exatamente o padrão dos componentes `src/components/ColorPicker.tsx` e `src/components/IconPicker.tsx`. Usar ícones relacionados a matérias escolares no IconPicker (ex.: `MdBook`, `MdScience`, `MdCalculate`, `MdLanguage`, `MdHistoryEdu`, `MdSportsFootball`, `MdMusicNote`, `MdPalette`, `MdComputer`, `MdBiotech`).
- Exibir a cor e o ícone na listagem de matérias.

---

## 6. Configurações — adicionar toggles de módulos

Implementar exatamente o que está descrito em `feature-toggles.prompt.md`:
- Adicionar colunas `usar_turmas`, `usar_professores`, `usar_frequencia`, `usar_recuperacao` em `configuracoes` (backend).
- Atualizar `models.rs`, `configuracoes.rs`, `types/index.ts`.
- Adicionar seção "Módulos ativos" na página de configurações com checkboxes para cada um dos quatro módulos.

---

## 7. Notas — corrigir erro ao salvar

Investigar e corrigir o bug em que clicar em "Salvar" na página de notas não faz nada e exibe um tooltip vazio no campo "valor".

Passos:
1. Ler `src/app/(main)/notas/page.tsx` — verificar validação do campo `valor` (se há validação de mínimo/máximo ou tipo que esteja bloqueando silenciosamente).
2. Verificar se o `invokeCmd` de criação de nota está enviando todos os campos obrigatórios com os nomes corretos.
3. Ler `src-tauri/src/notas.rs` — conferir parâmetros do comando `create_nota`.
4. Corrigir a discrepância e garantir que erros do backend sejam exibidos via `Toast`.

---

## 8. Cronograma — mostrar turma ou aluno, e ponteiro de hora atual

**Campo turma/aluno:**
- Quando `usar_turmas === true`: exibir select de turma no formulário de evento.
- Quando `usar_turmas === false`: exibir campo de seleção de aluno (multi-select, opcional) no formulário de evento.
- Atualizar backend: `ALTER TABLE eventos ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL;` e `ALTER TABLE eventos ADD COLUMN aluno_ids TEXT NOT NULL DEFAULT '[]';` (JSON array de ids).

**Ponteiro de hora atual:**
- Na visualização de grade (grid semanal), exibir uma linha horizontal vermelha na posição correspondente à hora atual.
- Destacar visualmente a coluna do dia de hoje (fundo levemente diferente ou cabeçalho em destaque).
- Atualizar o ponteiro a cada minuto via `setInterval`.

---

## 9. Dashboard — aulas de hoje

Adicionar uma seção na página do dashboard chamada "Aulas de Hoje":
- Buscar os eventos do cronograma cujo `dia_semana` corresponde ao dia atual da semana.
- Exibir em formato de grade ou lista cronológica, com hora de início, hora de fim, título do evento, e turma/aluno conforme o módulo ativo.
- Se não houver aulas, exibir mensagem "Nenhuma aula hoje."
- Reutilizar a lógica de busca de eventos já existente em `src/app/(main)/cronograma/page.tsx`.

---

## Ordem de execução recomendada

1. Backend: migrations de todas as tabelas novas/colunas novas (DB só pode ser migrado uma vez por versão).
2. Backend: atualizar structs em `models.rs`.
3. Backend: corrigir comandos existentes (matérias, notas, turmas, alunos) e criar novos (professores, configurações).
4. Frontend: tipos (`types/index.ts`).
5. Frontend: corrigir páginas com bugs (turmas, alunos, matérias, notas).
6. Frontend: melhorias de UX (provas, cronograma, dashboard, professores, configurações).
