
"professores" precisa de mais campos, mais relevantes, atualmente só tem o email, mas precisava de mais alguns, inclusive relacionando o professor às turmas (quando o usuário optar por mostra-las), relacionando a matérias (que o professor leciona - podendo ser uma ou VÁRIAS), aulas por semana, cronograma de aulas pessoal do professor (ao estilo da página de cronogramas)... seja criativo, atualmente está MUITO VAZIO.


---

Ainda está dando:

"Erro ao salvar."

em Turmas

---

Em PROVAS ainda está aparecendo o campo "Margens" que não serve pra absolutamente NADA!!!!!!!! aquilo só está ali para ocupar espaço desnecessário

essa funcionalidade de QR Code é meio desnecessária (totalmente desnecessária), pode remover

a página de provas precisava de uma UX-UI melhor, por exemplo pra filtrar provas por matéria, por semestre, por categoria (recuperação ou não)... seja criativo... não se limite a apenas melhorar os filtros, proponha uma UX muito melhor

---

em alunos está dando o mesmo erro, e precisa da seleção de arquivos para mandar uma foto do aluno, não é copiar o caminho!!!!!!!

essas imagens devem ser todas salvas em um lugar seguro dentro dos arquivos do app final (quando der build e instalar em um computador de fato) - qualquer imagem salva pelo usuário deve ser salva assim


---

em matéria:

1. toda vez que clico para salvar, dá esse erro:

Erro ao salvar matéria:"invalid args `cargaHorariaSemanal` for command `create_materia`: command create_materia missing required key cargaHorariaSemanal"

2. deve haver um seletor de cor e um seletor de icone (react-icons) seguindo os padrões dos dois componentes #file:ColorPicker.tsx  e #file:IconPicker.tsx  (pode copíar igualzinho, só mudar os icones mesmo para ser algo que tenha a ver com matérias escolares)

---

pensei que você iria adicionar na página de configurações as opcionalidades que combinamos, que são:

usar_turmas
usar_professores
usar_frequencia
usar_recuperacao

está dentro de #file:feature-toggles.prompt.md 

---

em Notas, não estou conseguindo salvar uma nova nota, toda vez que clico em "salvar" aparece um tooltip vazio sobre o campo "valor" e nada acontece

---

Em "cronograma" precisa aparecer "turma", quando o usuário tiver optado por mostrar as funcionalidades, e quando não tiver, precisa aparecer "aluno" - podendo ser mais de um aluno (campo opcional)


precisa aparecer um ponteiro mostrando o dia atual e a hora atual também na visualização de grade

No dashboard deve aparecer uma área semelhante à grade do cronograma, só que mostrando as aulas do dia de hoje

---

