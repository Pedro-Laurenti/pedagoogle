---
name: Pedagoogle project context
description: Key architectural decisions and recent features implemented in the Pedagoogle school management app
type: project
---

Pedagoogle is a Tauri 2.0 (Rust + Next.js) desktop app for school management (grades, schedule, exams, PDF/Word export).

**Architecture:**
- Backend: Rust (src-tauri/src/), SQLite via rusqlite, PDF via Typst engine, Word via docx-rs
- Frontend: Next.js/React with DaisyUI + TailwindCSS, icons from react-icons/md only
- Commands: snake_case Rust → camelCase TS (Tauri auto-converts)
- DB migrations in db.rs MIGRATIONS array (currently up to migration 69)

**Key modules:** alunos, materias, turmas, provas, notas, cronograma, atividades, configuracoes, typst_pdf, word

**Recent features added (2026-03-30):**
- Fixed updater: now uses GitHub API (api.github.com/repos/.../releases/latest) instead of redirect URL
- Boletim PDF/Word: professional 4-bimestre table with grades and absences per subject, landscape A4
- Attendance (faltas): new `faltas` table (migration 69), get_faltas_aula/save_faltas_aula/get_faltas_por_materia commands, attendance modal in cronograma page
- Backup: backup_completo() creates timestamped folder with DB+images, restore_from_backup() uses ATTACH for smart restore
- export_boletim_pdf now requires `ano_letivo` parameter; export_boletim_word added

**Why:** User needed proper 4-bimestre school report cards, per-class attendance tracking, and a reliable backup system before updates.
**How to apply:** When working on export features or adding new fields to boletim, remember the `get_nota_bimestre` pub(crate) helper in typst_pdf.rs and the `faltas` table for attendance counts.
