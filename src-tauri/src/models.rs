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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Materia {
    pub id: i64,
    pub nome: String,
    pub descricao: String,
    pub professor: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Aluno {
    pub id: i64,
    pub nome: String,
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Nota {
    pub id: i64,
    pub aluno_id: i64,
    pub prova_id: Option<i64>,
    pub descricao: String,
    pub valor: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Aula {
    pub id: i64,
    pub materia_id: Option<i64>,
    pub dia_semana: String,
    pub hora_inicio: String,
    pub hora_fim: String,
}

