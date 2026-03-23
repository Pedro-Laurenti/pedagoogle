Ordem recomendada considerando as dependências entre prompts:

| # | Prompt | Motivo |
|---|---|---|
| 1 | **regras-negocio** | Só lógica/validações, sem esquema novo — menor risco |
| 2 | **professores** | Cria tabela `professores`; `alunos-materias` depende disso para `professor_id` em matérias |
| 3 | **alunos-materias** | Usa `professores` (item 2); adiciona `cor` nas matérias, que `cronograma-extra` vai consumir |
| 4 | **configuracoes-extra** | Adiciona `nota_minima` nas configs — necessário antes do boletim PDF de `notas-extra` |
| 5 | **feature-toggles** | Depende de `professores` (item 2) e `configuracoes` estabilizados; adiciona checkboxes para ativar/desativar módulos (turmas, professores, frequência, recuperação) — suporte a homeschooling e escolas simples |
| 6 | **cronograma-extra** | Depende de `materia.cor` adicionado em `alunos-materias` |
| 7 | **notas-extra** | Depende de `turma_id` nas provas e `nota_minima` das configs (item 4) |
| 8 | **recuperacao-frequencia** | Depende da estrutura de notas e aulas estável; `nota_minima` já deve existir; as rotas criadas aqui já terão `feature` definido pelo item 5 |
| 9 | **provas-banco** | Independente — nova tabela `banco_questoes`, sem dependências externas |
| 10 | **pdf-extra** | Só adiciona campos nas provas, independente |
| 11 | **dashboard** | Lê de todas as tabelas — melhor rodar quando a estrutura está completa |
| 12 | **db-arquitetura** | Grande refactor (migrações numeradas + Mutex) — deixar por último quando tudo está funcionando |