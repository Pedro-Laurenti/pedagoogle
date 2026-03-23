---
name: cronograma-extra
description: Visualização semanal em grade e semestre letivo no cronograma
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Visualização semanal em grade (🟡)

`src/app/(main)/cronograma/page.tsx`:
- Adicionar toggle "Grade / Lista" no cabeçalho
- No modo Grade: renderizar grid CSS com:
  - Colunas = dias da semana (`["Seg", "Ter", "Qua", "Qui", "Sex"]`)
  - Linhas = intervalos de 50min de 07:00 a 22:00 (gerados dinamicamente)
  - Cada aula é posicionada na célula `(dia_semana, hora_inicio)` e ocupa altura proporcional à duração
  - Cor de fundo da célula = `materia.cor` (campo adicionado em `alunos-materias.prompt.md`); usar `#6366f1` como fallback
  - Célula exibe `materia_nome` e `hora_inicio–hora_fim`
- No modo Lista: manter a tabela atual
- State: `const [modoGrade, setModoGrade] = useState(true)`

> Depende de `materia.cor` — implementar após `alunos-materias.prompt.md`.

## 2. Semestre letivo (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE aulas ADD COLUMN semestre TEXT NOT NULL DEFAULT '2026-1';
```

**Backend** (`cronograma.rs`):
- `list_aulas`: filtrar por semestre quando passado; adicionar parâmetro `semestre: Option<String>` (None = retorna todos)
- `create_aula` / `update_aula`: incluir `semestre`
- Novo comando `copy_semestre(de: String, para: String) -> Result<i64, String>` — duplica todas as aulas do semestre `de` com `semestre = para`; retorna quantidade copiada

**Frontend** (`cronograma/page.tsx`):
- Select "Semestre" acima da grade (lista os semestres distintos presentes em `aulas` + opção "Novo")
- Botão `MdCopyAll` "Copiar para novo semestre" — abre dialog com campo "Semestre destino" e chama `copy_semestre`
- Campo `semestre` no modal de criação de aula (valor padrão = semestre selecionado)
