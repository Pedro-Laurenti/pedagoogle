use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Configuracoes {
    pub nome_escola: String,
    pub logo_path: String,
    pub cidade: String,
    pub diretor: String,
    pub moldura_estilo: String,
    pub margem_folha: f64,
    pub margem_moldura: f64,
    pub margem_conteudo: f64,
    pub fonte: String,
    pub nota_minima: f64,
    pub ano_letivo: String,
    pub tamanho_fonte: i64,
    pub tema: String,
    pub usar_turmas: bool,
    pub usar_professores: bool,
    pub usar_frequencia: bool,
    pub usar_recuperacao: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Professor {
    pub id: i64,
    pub nome: String,
    pub email: String,
    pub telefone: String,
    pub especialidade: String,
    pub aulas_por_semana: i64,
    pub observacoes: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfessorCronograma {
    pub id: i64,
    pub professor_id: i64,
    pub titulo: String,
    pub dia_semana: i64,
    pub hora_inicio: String,
    pub hora_fim: String,
    pub cor: String,
    pub recorrente: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfessorCronogramaInput {
    pub titulo: String,
    pub dia_semana: i64,
    pub hora_inicio: String,
    pub hora_fim: String,
    pub cor: String,
    pub recorrente: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Materia {
    pub id: i64,
    pub nome: String,
    pub descricao: String,
    pub professor_id: Option<i64>,
    pub professor_nome: Option<String>,
    pub turma_id: Option<i64>,
    pub turma_nome: Option<String>,
    pub carga_horaria_semanal: i64,
    pub cor: String,
    pub icone: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Turma {
    pub id: i64,
    pub nome: String,
    pub ano_letivo: String,
    pub turno: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Aluno {
    pub id: i64,
    pub nome: String,
    pub turma_id: Option<i64>,
    pub matricula: String,
    pub turma_nome: Option<String>,
    pub foto_path: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlunoCsvRow {
    pub nome: String,
    pub matricula: String,
    pub turma_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Prova {
    pub id: i64,
    pub titulo: String,
    pub descricao: String,
    pub materia_id: Option<i64>,
    pub data: String,
    pub rodape: String,
    pub margens: String,
    pub valor_total: f64,
    pub escola_override: String,
    pub cidade_override: String,
    pub turma_id: Option<i64>,
    pub is_recuperacao: bool,
    pub qr_gabarito: bool,
    pub duas_colunas: bool,
    pub paisagem: bool,
    pub updated_at: String,
    pub questoes_count: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Presenca {
    pub id: i64,
    pub aluno_id: i64,
    pub aula_id: i64,
    pub aluno_nome: String,
    pub data: String,
    pub presente: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FrequenciaMateria {
    pub materia_nome: String,
    pub total_aulas: i64,
    pub presencas: i64,
    pub percentual: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BancoQuestao {
    pub id: i64,
    pub tipo: String,
    pub enunciado: String,
    pub opcoes: String,
    pub valor: f64,
    pub tags: String,
    pub dificuldade: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Questao {
    pub id: i64,
    pub prova_id: i64,
    pub enunciado: String,
    pub tipo: String,
    pub opcoes: serde_json::Value,
    pub ordem: i64,
    pub valor: f64,
    pub linhas_resposta: i64,
    pub tags: String,
    pub dificuldade: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuestaoInput {
    #[serde(default)]
    pub id: Option<i64>,
    pub enunciado: String,
    pub tipo: String,
    pub opcoes: serde_json::Value,
    pub valor: f64,
    pub linhas_resposta: i64,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub dificuldade: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CategoriaLancamento {
    pub id: i64,
    pub nome: String,
    pub cor: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Nota {
    pub id: i64,
    pub aluno_id: i64,
    pub prova_id: Option<i64>,
    pub descricao: String,
    pub valor: f64,
    pub updated_at: String,
    pub categoria_id: Option<i64>,
    pub categoria_nome: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Aula {
    pub id: i64,
    pub materia_id: Option<i64>,
    pub dia_semana: String,
    pub hora_inicio: String,
    pub hora_fim: String,
    pub semestre: String,
    pub turma_id: Option<i64>,
    pub aluno_ids: String,
    pub bimestre: i64,
}

