---
name: melhorias-gerais
description: Corrige todos os bugs e melhorias listados em MELHORIAS.md
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Objetivo

Corrigir todos os bugs e implementar todas as melhorias listadas abaixo. Siga cada seção na ordem.

## Problemas estruturais

1. Quando executo o instalador, até dá certo e o aplicativo é iniciado no windows, porém aparece uma notificação que esse app não é seguro, queria que desse pra instalar sem passar por isso... parece que o app é inseguro sendo que na verdade é muito simples

2. Quando eu instalo e abro o app, fica uma janela preta (prompt) rodando de fundo, se eu tento fechar essa janela, fecha a janela do app também (isso só acontece quando instalo o app, rodando em dev não aparece nada)

3. Eu queria customizar a logo do meu app e deixa-la disponivel em todos os lugares (barra de tarefas, icones, tudo mesmo) - atualmente está um círculo roxo... é só dar replace nessas imagens e lançar uma nova versão?

4. Deve existir uma ferramenta interna que busque por novas atualizações do app e faça o update do app para a versão mais recente. essa opção deve ficar na página de configurações. Essa ferramenta deve tomar um cuidado para fazer a portabilidade dos dados no banco de dados, sem perder nenhum dado retroativo das versões anteriores.

## Problemas no frontend
1. agrupe todas as páginas (incluindo as novas páginas que vão ser criadas mais pra frente) em grupos colapsáveis no navigation (navbar/sidebar).

2. Quero que TODOS OS INPUTS sejam componentes padronizados - todos precisam seguir o estilo de inputs do daisyUI - TODO O PROJETO DEVE USAR ESSES COMPONENTES, NENHUM INPUT HARDCODED É PERMITIDO. Quero criar os seguintes componentes:
    - Número de telefone com máscara (padrão brasileiro de "(00) 00000-0000") e validação caso não preencha tudo
    - E-mail com validações
    - CPF com máscara (padrão brasileiro de "000.000.000-00") e validação caso não preencha tudo
    - Multi-select com tags — é um campo de seleção que permite escolher múltiplos valores de uma lista. Ao invés de substituir a seleção anterior a cada clique (como um <select> comum), cada valor escolhido é acumulado e exibido como uma tag (badge) ao lado do campo. Cada tag possui um botão × para remoção individual.
    - INPUT DE DATAS - é um input usando alguma biblioteca react que padroniza calendário para português brasileiro (inclusive com um picker de datas) e apresente a data em formato dd/mm/aaaa
    - INPUT DE HORAS - um input que padroniza um picker de horas, podendo ser configurado
    - input de rich text já existe, basta padronizar com o mesmo estilo dos demais
    - INPUT DE IMAGENS FUNCIONAL
    - padronizar também demais inputs simples:
        - checkboxes
        - radioboxes
        - texto simples
        - caixa de texto simples

3. criar um componente "modal", que deverá ser utilizado em TODAS AS CONFIRMAÇÕES DO SISTEMA, esse componente pode ser configurado via parametros pra ser:
- modal com botões de confirmação
- modal simples de informação
- modal com conteúdo complexo (por exemplo, um modal que contenha um formulário)
- as cores desses modais podem ser personalizaveis (cores-padrão do daisyUI)

### correções página por página:

#### Dashboard
1. Precisa ser mais intuitivo, apresentar shortcuts para as páginas principais, apresentar mais icones e talvez algum gráfico

#### Professores
1. os inputs de número de celular, e-mail e observações podem ser excluidos, são totalmente desnecessário
2. "especialidade" deve ser alterada pra "matéria", onde terá uma selecbox com todas as matérias do sistema, podendo ser selecionada mais de uma matéria usando o novo componente "Multi-select"
3. devem ser adicionados alguns filtros nessa página também, respeitando as preferências do usuário nos "usar_..."
4. todos os inputs devem ser padronizados usando os componentes de input
5. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
6. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Página Matérias
1. quando o usuário tiver o "usar_turmas" como false, então o filtro por turmas deve sumir
2. o modal de adição está ocupando muita área vertical, quero que você divida em alguns steps para que evite de ter um scroll em um modal, de modo que tudo caiba em uma tela só
3. dentro do modal de edição/criação de matérias, se o usuário tiver "usar_turmas" como false, então o campo de turma deve sumir
4. dentro do modal de edição/criação de matérias, se o usuário tiver "usar_professores" como false, então o campo de professores deve sumir
5. "descrição" não precisa existir nesse formulário, é totalmente desnecessário
6. melhore a UX-UI de "Aulas/Semanas" para ser um elemento interativo... Onde é mostrada a carga horária semanal, quantos horários livres atualmente existem (tanto por dia, quanto por semana e por mês) e quais são as alocações disponíveis.

Para que esse ponto 6 dê certo, é necessário adicionar na aba de configuração uma sessão de configurar a carga horária de funcionamento de cada turno, por exemplo: 6 aulas por dia, 45 minutos por aula - logo poderão ser feitas N horas por dia, configurando o horário de entrada poderá se chegar ao horário de encerramento das aulas - também poderemos saber quantas horas/aulas por semana, quantas horas/aulas por mês, quantas horas/aulas por bimestre, quantas horas/aulas por semestre, quantas horas/aulas por ano, etc

7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Página Provas
1. na página principal de provas, os filtros devem ser editados:
    - Inputs devem ser os padronizados pelos componentes de input do frontend
2. você ignorou a regra de NUNCA USAR EMOJIS, e usou emji no card das provas, use a biblioteca REACT-ICONS ao invés de emojis
3. alguns campos devem ser obrigatórios, e se o usuário não tiver preenchido tudo, deve aparecer um toast e os campos faltantes devem ficar com bordas vermelhas:
    - Título
    - Matéria
    - Data
    - Valor total da Prova
4. o campo "turma" só deve aparecer se o usuário tiver "usar_turmas" como true
5. "Instruções/Descrição" deve ser um campo richtext - deve ser implementado também no backend para que seja richtext
6. o botão "exportar versão" não faz sentido existir, pode excluir tanto do frontend quanto do backend
7. Implemente um fluxo de steps ou abas, a primeira são as configurações da prova, a segunda são as questões
8. o conteúdo deve ficar contido dentro das molduras, respeitando as regras definidas em "config", atualmente o conteúdo está ultrapassando a moldura e deixando tudo bagunçado
9. o cabeçalho não está seguindo o padrão de cabeçalho que idealizei... precisava ser formatado assim (esse exemplo é ilustrativo - MAS FICOU MUITO BONITO!):

```html
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>Cabeçalho Escolar</title>
  <link href="https://fonts.googleapis.com/css2?family=Playfair+Display:wght@600;700&family=Crimson+Pro:ital,wght@0,400;0,600;1,400&display=swap" rel="stylesheet"/>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
 
    :root {
      --azul-escola:  #1a3557;
      --azul-claro:   #2a5298;
      --dourado:      #c9a84c;
      --cinza-linha:  #b0bec5;
      --cinza-texto:  #37474f;
      --branco:       #ffffff;
      --fundo:        #f4f6f8;
    }
 
    body {
      background: var(--fundo);
      display: flex;
      justify-content: center;
      align-items: flex-start;
      min-height: 100vh;
      padding: 40px 20px;
      font-family: 'Crimson Pro', Georgia, serif;
    }
 
    /* ── FOLHA ── */
    .folha {
      background: var(--branco);
      width: 100%;
      max-width: 860px;
      border-radius: 4px;
      box-shadow: 0 2px 16px rgba(0,0,0,.12);
      overflow: hidden;
    }
 
    /* ── FAIXAS DECORATIVAS ── */
    .faixa-topo   { background: var(--azul-escola); height: 6px; }
    .faixa-dourada{ background: var(--dourado);     height: 3px; }
    .faixa-base   { background: var(--azul-escola); height: 4px; }
 
    /* ── CABEÇALHO PRINCIPAL ── */
    .cabecalho {
      display: flex;
      align-items: center;
      padding: 20px 28px 18px;
      border-bottom: 2px solid var(--azul-escola);
    }
 
    /* LOGO — 2/10 */
    .logo-wrapper {
      flex: 0 0 20%;
      display: flex;
      justify-content: center;
      align-items: center;
      padding-right: 20px;
      border-right: 2px solid var(--dourado);
    }
 
    .logo-placeholder {
      width: 96px;
      height: 96px;
      border-radius: 50%;
      border: 3px solid var(--azul-escola);
      background: linear-gradient(135deg, var(--azul-escola) 0%, var(--azul-claro) 100%);
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 5px;
      color: var(--branco);
      text-align: center;
      position: relative;
      overflow: hidden;
    }
 
    .logo-placeholder::before {
      content: '';
      position: absolute;
      inset: 0;
      background: radial-gradient(circle at 35% 35%, rgba(255,255,255,.18) 0%, transparent 65%);
    }
 
    .logo-placeholder svg {
      width: 38px;
      height: 38px;
      opacity: .92;
      position: relative;
    }
 
    .logo-placeholder .logo-label {
      font-size: 9.5px;
      font-weight: 700;
      letter-spacing: .1em;
      text-transform: uppercase;
      line-height: 1.3;
      position: relative;
      opacity: .85;
    }
 
    /* INFO — 8/10 */
    .info-wrapper {
      flex: 0 0 80%;
      padding-left: 22px;
    }
 
    .escola-nome {
      font-family: 'Playfair Display', serif;
      font-size: 17.5px;
      font-weight: 700;
      color: var(--azul-escola);
      letter-spacing: .02em;
      line-height: 1.25;
      margin-bottom: 2px;
    }
 
    .escola-cidade {
      font-size: 12.5px;
      color: var(--cinza-texto);
      font-style: italic;
      margin-bottom: 13px;
    }
 
    /* GRADE DE CAMPOS */
    .grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px 20px;
    }
 
    .campo {
      display: flex;
      align-items: baseline;
      gap: 5px;
      font-size: 13.5px;
      color: var(--cinza-texto);
    }
 
    .campo.full  { grid-column: 1 / -1; }
 
    .label {
      font-size: 11.5px;
      font-weight: 600;
      color: var(--azul-escola);
      text-transform: uppercase;
      letter-spacing: .06em;
      white-space: nowrap;
      flex-shrink: 0;
    }
 
    input.linha {
      flex: 1;
      min-width: 28px;
      border: none;
      border-bottom: 1.5px dotted var(--cinza-linha);
      background: transparent;
      outline: none;
      font-family: 'Crimson Pro', Georgia, serif;
      font-size: 13.5px;
      color: var(--cinza-texto);
      padding: 0 2px 1px;
      transition: border-color .2s;
    }
    input.linha:focus { border-bottom-color: var(--azul-claro); }
    input.linha::placeholder { color: transparent; }
 
    /* tamanhos fixos para campos específicos */
    .w-dia  { flex: 0 0 32px;  min-width: 32px; }
    .w-mes  { flex: 0 0 110px; min-width: 110px; }
    .w-ano  { flex: 0 0 48px;  min-width: 48px; }
    .w-sm   { flex: 0 0 50px;  min-width: 50px; }
    .w-md   { flex: 0 0 80px;  min-width: 80px; }
    .w-val  { flex: 0 0 60px;  min-width: 60px; }
 
    .sep {
      grid-column: 1 / -1;
      border: none;
      border-top: 1px solid #e0e6ec;
      margin: 2px 0;
    }
 
    .txt { white-space: nowrap; font-size: 13px; color: var(--cinza-texto); }
  </style>
</head>
<body>
 
<div class="folha">
  <div class="faixa-topo"></div>
  <div class="faixa-dourada"></div>
 
  <div class="cabecalho">
 
    <!-- ░░ LOGO 2/10 ░░ -->
    <div class="logo-wrapper">
      <div class="logo-placeholder">
        <svg viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <polygon points="24,7 46,18 24,29 2,18" fill="rgba(255,255,255,0.92)"/>
          <path d="M11 23v9c0 5.5 6 9.5 13 9.5s13-4 13-9.5V23"
                stroke="rgba(255,255,255,0.78)" stroke-width="2.2"
                fill="none" stroke-linejoin="round"/>
          <line x1="46" y1="18" x2="46" y2="33"
                stroke="rgba(255,255,255,0.7)" stroke-width="2.5" stroke-linecap="round"/>
          <circle cx="46" cy="34" r="2.2" fill="rgba(255,255,255,0.7)"/>
        </svg>
        <span class="logo-label">Logo<br>Escola</span>
      </div>
    </div>
 
    <!-- ░░ INFO 8/10 ░░ -->
    <div class="info-wrapper">
 
      <div class="escola-nome">Nome Completo da Escola</div>
      <div class="escola-cidade">Cidade, Estado &mdash; Secretaria Municipal de Educação</div>
 
      <div class="grid">
 
        <!-- Data -->
        <div class="campo full">
          <span class="label">Data:</span>
          <span class="txt">Inhumas,</span>
          <input class="linha w-dia"  type="text" maxlength="2" />
          <span class="txt">de</span>
          <input class="linha w-mes"  type="text" />
          <span class="txt">de</span>
          <input class="linha w-ano"  type="text" maxlength="4" />
        </div>
 
        <hr class="sep"/>
 
        <!-- Professor -->
        <div class="campo full">
          <span class="label">Professor(a):</span>
          <input class="linha" type="text" />
        </div>
 
        <!-- Ano | Turma -->
        <div class="campo">
          <span class="label">Ano:</span>
          <input class="linha w-sm" type="text" maxlength="3"/>
          <span class="label" style="margin-left:10px">Turma:</span>
          <input class="linha w-sm" type="text" maxlength="4"/>
        </div>
 
        <!-- Turno -->
        <div class="campo">
          <span class="label">Turno:</span>
          <input class="linha" type="text"/>
        </div>
 
        <!-- Valor | Nota -->
        <div class="campo">
          <span class="label">Valor:</span>
          <input class="linha w-val" type="text"/>
        </div>
 
        <div class="campo">
          <span class="label">Nota:</span>
          <input class="linha w-val" type="text"/>
        </div>
 
      </div><!-- /grid -->
    </div><!-- /info-wrapper -->
 
  </div><!-- /cabecalho -->
 
  <div class="faixa-dourada"></div>
  <div class="faixa-base"></div>
</div>
 
</body>
</html>
```

10. o TÍTULO DA PROVA não precisa ser editavel pelo usuário, ele deve ser SEMPRE:
```
AVALIAÇÃO DE [NOME DA MATÉRIA] - [Nº DO BIMESTRE] BIMESTRE 
```
11. Não precisa ter o campo "data" apenas deve ter um campo definindo qual bimestre de qual ano, o aluno deverá completar o mês e o dia de realização

12. remova as opções (tanto backend quanto frontend) de "é prova de recuperação", de "layout duas colunas" e de "orientação paisagem", isso não é necessário

13. todos os inputs devem ser padronizados usando os componentes de input
14. Nem sempre será uma questão, uma prova pode conter um texto também, adicione um botão de adicionar texto sem estar relacionado a questões.
15. Questões devem conter uma confuiguração de espaço a ser saltado (tamanho do espaço pode variar a depender do usuário) para usar como rascunho de contas, esse espaço deve ser vazio, sem linhas... mas pode ser contabilizado (para melhor entendimento do usuário) como "linhas em branco" e usar o mesmo espaçamento de uma linha, só não colocando linhas - TODAS AS QUESTÕES PODEM TER ESSES ESPAÇOS
16. remova a função de banco de questões, isso é desnecessário (tanto backend quanto frontend)
17. remova a função de tags em cada questão, isso é desnecessário (tanto backend quanto frontend)
18. remova a função de cabeçalho personalizado, isso é desnecessário (tanto backend quanto frontend)
19. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
20. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Página Atividades

Deverá existir uma nova página de atividades que será uma cópia da página de provas, só que com mais algumas funcionalidades, que serão:

Essa página deve copiar "provas" em tudo, exceto em:
1. título pode ser editável pelo usuário que escrever a atividade, mas deve aparecer sempre um pré-titulo:
```
[NOME DA MATÉRIA (pequeno)]
[TITULO DA ATIVIDADE (titulo definido pelo usuário escrito em uma fonte maior que o nome da matéria)]
```
2. Atividades podem ou não valer pontos, o usuário deve marcar uma checkbox "vale nota"? e se valer, aparecerá os campos no cabeçalho
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Turmas
1. deve existir uma lógica de relacionar uma turma com N matérias cadastradas, somente quando o usuário optar por "usar_turmas", esse atrelamento de matérias-turmas devem respeitar a configuração de carga horária definidos na página de configuração
2. "ano letivo" não precisa existir
3. "nome" também não precisa existir
4. deve apenas existir os campos
    - "ano" (tipo a série, 6º ano, 8º ano, 1º do ensino médio, etc)
    - "turma" (letras do "A" ao "Z")
5. quando o usuário não quiser usar "turmas" deve ser algo não obrigatório para os outros formulários
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Alunos
1. quando o usuário optar por não usar turmas, aparecerá um campo obrigatório de relacionamento de matérias-aluno (um aluno pode ter várias matérias), devem respeitar a configuração de carga horária (quantidade de aulas no dia) definidos na página de configuração
2. o botão "importar CSV" é desnecessário, exclua tanto o frontend quanto o backend
3. dentro do formulário:
    - input de imagens não está funcionando, e é necessário usar o novo componente padronizado de input de imagens
    - o campo "turma" só deve aparecer se o usuário setar "usar_turmas" como true
    - o campo "matrícula" não é necessário, mas cada aluno deve ter um ID único
    - Nome está correto
4. quando o usuário optar por não usar turmas, o filtro de turmas deve desaparecer
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado


#### Notas
1. devem ser adicionados filtros que fazem sentido, observando os bimestres, anos, turmas (quando aplicaveis), matérias e só por fim, alunos
2. dentro do formulário
    - se o usuário optar por usar turmas, deve ter uma select com as turmas primeiro, pra depois carregar a select dos alunos
    - "categoria" deve englobar o campo abaixo - "prova", pois prova é uma categoria... logo deve ter uma configuração dentro do modal de categorias que atrele uma nova categoria ao sistema de provas do sistema
        - se o usuário selecionar a categoria "prova", então deve aparecer uma selectbox para matéria, bimestre, ano, e só depois lista as provas que passarem por essa seleção
    - pode remover o campo "descrição", é desnecessário (tanto backend quanto frontend)
    - campo valor não pode passar o valor de uma matéria no bimestre
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Cronograma
1. apenas podemos adicionar uma aula durante a carga horária (definida nas configurações)
2. podemos adicionar uma aula com um click
3. no formulário, podemos adicionar uma recorrência usando o input multi-select (padronizado nos componentes)
4. as opções de semestre e bimestre podem desaparecer, é desnecessário configurar isso
5. "turma" deve desaparecer se o usuário não estiver com "usar_turmas" como "true"
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado

#### Frequência
Exclua essa página totalmente, não precisamos dessa funcionalidade no sistema


#### Configurações
1. a scroll de páginas grandes está bugada, quando o conteúdo é maior que a tela cria DUAS SCROLLS, só deve existir a interna... lembrando que a sidebar deve ser sempre fixed
7. todos os inputs devem ser padronizados usando os componentes de input
8. os modais de edição e criação devem utilizar o novo componente de "modal" padronizado
9. deve ter um modal de confirmação de exclusão, também utilizando o novo componente de "modal" padronizado