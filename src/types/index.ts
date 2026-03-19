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
}

export interface Materia {
  id: number;
  nome: string;
  descricao: string;
  professor: string;
}

export interface Aluno {
  id: number;
  nome: string;
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
}

export interface QuestaoInput {
  id?: number;
  tempId?: number;
  enunciado: string;
  tipo: TipoQuestao;
  opcoes: OpcaoQuestao[];
  valor: number;
  linhas_resposta: number;
}

export interface Nota {
  id: number;
  aluno_id: number;
  prova_id: number | null;
  descricao: string;
  valor: number;
}

export interface Aula {
  id: number;
  materia_id: number | null;
  dia_semana: string;
  hora_inicio: string;
  hora_fim: string;
}

export interface ToastState {
  message: string;
  type: "success" | "error" | "warning" | "info";
}

export interface Aluno {
  id: number;
  nome: string;
}

export interface Prova {
  id: number;
  titulo: string;
  descricao: string;
  materia_id: number | null;
  data: string;
  cabecalho: string;
  rodape: string;
  margens: string;
}

export interface Questao {
  id: number;
  prova_id: number;
  enunciado: string;
  tipo: TipoQuestao;
  opcoes: OpcaoQuestao[];
  ordem: number;
}

export interface QuestaoInput {
  id?: number;
  tempId?: number;
  enunciado: string;
  tipo: TipoQuestao;
  opcoes: OpcaoQuestao[];
}

export interface Nota {
  id: number;
  aluno_id: number;
  prova_id: number | null;
  descricao: string;
  valor: number;
}

export interface Aula {
  id: number;
  materia_id: number | null;
  dia_semana: string;
  hora_inicio: string;
  hora_fim: string;
}

export interface ToastState {
  message: string;
  type: "success" | "error" | "warning" | "info";
}
