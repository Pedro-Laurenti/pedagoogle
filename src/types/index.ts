export type TipoQuestao =
  | "dissertativa"
  | "multipla_escolha"
  | "verdadeiro_falso"
  | "completar_lacunas"
  | "associacao"
  | "ordenar"
  | "letras";

export interface OpcaoQuestao {
  texto: string;
  correta: boolean;
  par?: string;
  linhas?: number;
}

export type MolduraEstilo = 'none' | 'simple' | 'double' | 'ornate' | 'classic' | 'modern';

export interface Configuracoes {
  nome_escola: string;
  logo_path: string;
  cidade: string;
  estado: string;
  diretor: string;
  moldura_estilo: MolduraEstilo;
  margem_folha: number;
  margem_moldura: number;
  margem_conteudo: number;
  fonte: string;
  nota_minima: number;
  ano_letivo: string;
  tamanho_fonte: number;
  tema: string;
  usar_turmas: boolean;
  usar_professores: boolean;
  usar_frequencia: boolean;
  usar_recuperacao: boolean;
  aulas_por_dia: number;
  minutos_por_aula: number;
  hora_entrada: string;
  dias_letivos_semana: number;
}

export interface Professor {
  id: number;
  nome: string;
  aulas_por_semana: number;
}

export interface ProfessorCronograma {
  id: number;
  professor_id: number;
  titulo: string;
  dia_semana: number;
  hora_inicio: string;
  hora_fim: string;
  cor: string;
  recorrente: boolean;
}

export interface Materia {
  id: number;
  nome: string;
  professor_id: number | null;
  professor_nome?: string;
  turma_id: number | null;
  turma_nome?: string;
  carga_horaria_semanal: number;
  cor: string;
  icone: string;
}

export interface Turma {
  id: number;
  nome: string;
  ano: string;
  turma: string;
  turno: string;
}

export interface Aluno {
  id: number;
  nome: string;
  turma_id: number | null;
  turma_nome?: string;
  foto_path: string;
  updated_at: string;
}

export interface Prova {
  id: number;
  titulo: string;
  descricao: string;
  materia_id: number | null;
  bimestre: number;
  ano_letivo: string;
  valor_total: number;
  turma_id: number | null;
  updated_at: string;
  questoes_count: number;
}

export interface Atividade {
  id: number;
  titulo: string;
  descricao: string;
  materia_id: number | null;
  bimestre: number;
  ano_letivo: string;
  valor_total: number;
  turma_id: number | null;
  vale_nota: boolean;
  updated_at: string;
  questoes_count: number;
}

export interface AtividadeQuestao {
  id: number;
  atividade_id: number;
  enunciado: string;
  tipo: TipoQuestao | 'texto';
  opcoes: OpcaoQuestao[];
  ordem: number;
  valor: number;
  linhas_resposta: number;
  resposta: string;
  espaco_rascunho: number;
}

export interface BancoQuestao {
  id: number;
  tipo: string;
  enunciado: string;
  opcoes: string;
  valor: number;
  tags: string;
  dificuldade: string;
}

export interface Questao {
  id: number;
  prova_id: number;
  enunciado: string;
  tipo: TipoQuestao | 'texto';
  opcoes: OpcaoQuestao[];
  ordem: number;
  valor: number;
  linhas_resposta: number;
  resposta: string;
  espaco_rascunho: number;
}

export interface QuestaoInput {
  id?: number;
  tempId?: number;
  enunciado: string;
  tipo: TipoQuestao | 'texto';
  opcoes: OpcaoQuestao[];
  valor: number;
  linhas_resposta: number;
  resposta?: string;
  espaco_rascunho: number;
}

export interface CategoriaLancamento {
  id: number;
  nome: string;
  cor: string;
  vincula_provas: boolean;
}

export interface Nota {
  id: number;
  aluno_id: number;
  prova_id: number | null;
  valor: number;
  updated_at: string;
  categoria_id: number | null;
  categoria_nome: string | null;
}

export interface Aula {
  id: number;
  materia_id: number | null;
  dia_semana: string;
  hora_inicio: string;
  hora_fim: string;
  semestre: string;
  turma_id: number | null;
  aluno_ids: string;
  bimestre: number;
}

export interface ToastState {
  message: string;
  type: "success" | "error" | "warning" | "info";
}

export interface DashboardStats {
  total_provas: number;
  total_alunos: number;
  total_materias: number;
  total_notas: number;
  total_aulas: number;
}

export interface ProximaProva {
  id: number;
  titulo: string;
  data: string;
  materia_nome: string;
}

export interface MediaMateria {
  materia_nome: string;
  media: number;
}

export interface FaltaItem {
  aluno_id: number;
  aluno_nome: string;
  faltou: boolean;
}

export interface FaltasPorMateria {
  materia_id: number;
  materia_nome: string;
  bimestre: number;
  faltas: number;
}
