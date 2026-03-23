export type TipoQuestao =
  | "dissertativa"
  | "multipla_escolha"
  | "verdadeiro_falso"
  | "completar_lacunas"
  | "associacao"
  | "ordenar";

export interface OpcaoQuestao {
  texto: string;
  correta: boolean;
  par?: string;
}

export type MolduraEstilo = 'none' | 'simple' | 'double' | 'ornate' | 'classic' | 'modern';

export interface Configuracoes {
  nome_escola: string;
  logo_path: string;
  cidade: string;
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
  email: string;
  telefone: string;
  especialidade: string;
  aulas_por_semana: number;
  observacoes: string;
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
  descricao: string;
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
  ano_letivo: string;
  turno: string;
}

export interface Aluno {
  id: number;
  nome: string;
  turma_id: number | null;
  matricula: string;
  turma_nome?: string;
  foto_path: string;
  updated_at: string;
}

export interface AlunoCsvRow {
  nome: string;
  matricula: string;
  turma_id: number | null;
}

export interface Prova {
  id: number;
  titulo: string;
  descricao: string;
  materia_id: number | null;
  data: string;
  rodape: string;
  margens: string;
  valor_total: number;
  escola_override: string;
  cidade_override: string;
  turma_id: number | null;
  is_recuperacao: boolean;
  qr_gabarito: boolean;
  duas_colunas: boolean;
  paisagem: boolean;
  updated_at: string;
  questoes_count: number;
}

export interface Presenca {
  id: number;
  aluno_id: number;
  aula_id: number;
  aluno_nome: string;
  data: string;
  presente: boolean;
}

export interface FrequenciaMateria {
  materia_nome: string;
  total_aulas: number;
  presencas: number;
  percentual: number;
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
  tipo: TipoQuestao;
  opcoes: OpcaoQuestao[];
  ordem: number;
  valor: number;
  linhas_resposta: number;
  tags: string;
  dificuldade: string;
}

export interface QuestaoInput {
  id?: number;
  tempId?: number;
  enunciado: string;
  tipo: TipoQuestao;
  opcoes: OpcaoQuestao[];
  valor: number;
  linhas_resposta: number;
  tags: string;
  dificuldade: string;
}

export interface CategoriaLancamento {
  id: number;
  nome: string;
  cor: string;
}

export interface Nota {
  id: number;
  aluno_id: number;
  prova_id: number | null;
  descricao: string;
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
