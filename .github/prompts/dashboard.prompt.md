---
name: dashboard
description: Dashboard com estatísticas, alertas e médias reais
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## Backend
Criar `src-tauri/src/dashboard.rs` com quatro comandos:

1. `get_dashboard_stats` → `DashboardStats { total_provas: i64, total_alunos: i64, total_materias: i64 }` (SELECT COUNT(*) em cada tabela)
2. `list_proximas_provas` → `Vec<ProximaProva { id: i64, titulo: String, data: String, materia_nome: String }>` — provas com `data >= date('now')`, ORDER BY data ASC, LIMIT 5, LEFT JOIN materias
3. `get_alertas` → `Vec<String>` — lista de avisos:
   - "X prova(s) sem questões cadastradas" (provas sem rows em `questoes`)
   - "X aluno(s) sem nenhuma nota lançada" (alunos sem rows em `notas`)
4. `get_medias_por_materia` → `Vec<MediaMateria { materia_nome: String, media: f64 }>` — média das notas agrupadas por `materias.id`; LEFT JOIN provas ON notas.prova_id = provas.id; LEFT JOIN materias ON provas.materia_id = materias.id

Registrar todos em `lib.rs`.

## Frontend
`src/app/(main)/dashboard/page.tsx`:
- Carregar todos os quatro comandos no `useEffect`
- Substituir `"-"` pelo valor real em cada card de estatística
- Adicionar card "Próximas Provas" listando as provas retornadas com data e matéria
- Adicionar card "Alertas" com cada string de `get_alertas` como item de lista; se vazio, exibir "Nenhum alerta"
- Adicionar seção "Médias por Matéria": para cada item de `get_medias_por_materia`, exibir nome e barra CSS proporcional à média (sem dependência externa)

## Tipos
```ts
export interface DashboardStats { total_provas: number; total_alunos: number; total_materias: number; }
export interface ProximaProva { id: number; titulo: string; data: string; materia_nome: string; }
export interface MediaMateria { materia_nome: string; media: number; }
```
