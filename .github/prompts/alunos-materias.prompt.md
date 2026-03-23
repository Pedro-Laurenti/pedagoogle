---
name: alunos-materias
description: Filtros/busca/CSV em alunos e melhorias de matérias (turma, carga horária, cor)
model: Claude Sonnet 4.6 (copilot)
---

SIGA [RULES](../instructions/rules.instructions.md) | [FRONTEND](../instructions/frontend.instructions.md) | [BACKEND](../instructions/backend.instructions.md)

## 1. Filtro e busca em Alunos (🟡)

`src/app/(main)/alunos/page.tsx`:
- Adicionar `const [filtroNome, setFiltroNome] = useState("")` e `const [filtroTurma, setFiltroTurma] = useState<number | null>(null)`
- Acima da tabela: `<input>` de busca ("Buscar por nome ou matrícula") e `<select>` de turma (opções de `list_turmas` + opção "Todas")
- Filtrar client-side: `alunos.filter(a => (!filtroNome || a.nome.toLowerCase().includes(...) || a.matricula.includes(...)) && (!filtroTurma || a.turma_id === filtroTurma))`

## 2. Importação CSV de alunos (🟡)

**Backend** (`alunos.rs`) — dois novos comandos:
- `preview_import_alunos_csv(csv_content: String) -> Result<Vec<AlunoCsvRow>, String>` — parsear CSV simples (colunas: `nome,matricula,turma_id`); separar por `\n` e `,`; retornar Vec sem inserir
- `confirm_import_alunos(rows: Vec<AlunoCsvRow>) -> Result<i64, String>` — inserir todos e retornar contagem

**Modelo** (`models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AlunoCsvRow { pub nome: String, pub matricula: String, pub turma_id: Option<i64> }
```

**Frontend** (`alunos/page.tsx`):
- Botão `MdUploadFile` "Importar CSV"
- Abre modal com `<input type="file" accept=".csv">` — ao selecionar, ler com `FileReader`, chamar `preview_import_alunos_csv`
- Exibir tabela de preview (nome, matrícula, turma_id); botão "Confirmar" chama `confirm_import_alunos` e fecha modal

## 3. Foto do aluno (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE alunos ADD COLUMN foto_path TEXT NOT NULL DEFAULT '';
```

**Backend** (`alunos.rs`): incluir `foto_path` em `create_aluno`, `update_aluno`, `list_alunos`

**Frontend** (`alunos/page.tsx`):
- Campo "Foto (caminho)" no modal (input text)
- Na tabela, exibir `<img src={aluno.foto_path} className="w-8 h-8 rounded-full object-cover" />` se `foto_path` não vazio; caso contrário avatar placeholder `MdPerson`

## 4. Associar matéria a turma (🔴)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE materias ADD COLUMN turma_id INTEGER REFERENCES turmas(id) ON DELETE SET NULL;
```

**Backend** (`materias.rs`):
- `create_materia` / `update_materia`: incluir `turma_id: Option<i64>`
- `list_materias`: LEFT JOIN turmas para retornar `turma_nome: Option<String>`

**Frontend** (`materias/page.tsx`):
- Select `turma_id` no modal (opções de `list_turmas` + "Nenhuma")
- Coluna "Turma" na tabela
- Select de filtro por turma acima da tabela

**Tipos** (`types/index.ts`): em `Materia`, adicionar `turma_id: number | null; turma_nome?: string;`

## 5. Carga horária semanal (🟡)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE materias ADD COLUMN carga_horaria_semanal INTEGER NOT NULL DEFAULT 0;
```

**Backend** (`materias.rs`): incluir `carga_horaria_semanal` em create/update/list

**Frontend** (`materias/page.tsx`): input numérico `min="0"` "Aulas/semana" no modal

## 6. Cor da matéria (🟢)

**Backend** (`db.rs` → `migrate()`):
```sql
ALTER TABLE materias ADD COLUMN cor TEXT NOT NULL DEFAULT '#6366f1';
```

**Backend** (`materias.rs`): incluir `cor` em create/update/list

**Frontend** (`materias/page.tsx`): `<input type="color">` "Cor" no modal; exibir badge colorido na tabela  
**Uso**: cronograma/page.tsx pode usar `materia.cor` para colorir células da grade semanal
