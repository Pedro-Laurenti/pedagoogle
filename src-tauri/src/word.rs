use docx_rs::*;
use printpdf::image_crate as ic;
use crate::db::get_conn;
use crate::models::Questao;
use crate::html_render::{html_to_blocks, Block, Align};
use rusqlite::params;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Convert a parsed HTML Block into a docx Paragraph
fn block_to_paragraph(block: &Block, indent_twips: Option<i32>) -> Paragraph {
    match block {
        Block::Para { spans, align, heading } => {
            let sz: usize = match heading { Some(1) => 32, Some(2) => 28, Some(3) => 24, _ => 22 };
            let alignment = match align {
                Align::Center => Some(AlignmentType::Center),
                Align::Right  => Some(AlignmentType::Right),
                _             => None,
            };
            let mut p = Paragraph::new();
            if let Some(a) = alignment { p = p.align(a); }
            if let Some(ind) = indent_twips {
                p = p.indent(Some(ind), None, None, None);
            }
            for span in spans {
                if span.text.is_empty() { continue; }
                let mut run = Run::new().add_text(&span.text).size(sz);
                if span.bold    { run = run.bold(); }
                if span.italic  { run = run.italic(); }
                if span.underline { run = run.underline("single"); }
                p = p.add_run(run);
            }
            p
        }
        Block::Hr => par_hline(),
        Block::TableRow { cells, is_header } => {
            // Flatten table row cells as a single indented text line
            let txt = cells.iter().map(|cell| {
                cell.iter().filter_map(|b| if let Block::Para { spans, .. } = b {
                    Some(spans.iter().map(|s| s.text.as_str()).collect::<String>())
                } else { None }).collect::<String>()
            }).collect::<Vec<_>>().join("  |  ");
            let mut p = Paragraph::new();
            let mut run = Run::new().add_text(&txt).size(20);
            if *is_header { run = run.bold(); }
            p = p.add_run(run);
            if let Some(ind) = indent_twips { p = p.indent(Some(ind), None, None, None); }
            p
        }
    }
}

fn format_date_pt(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let dia: u32 = parts[2].parse().unwrap_or(0);
    let mes = match parts[1] {
        "01" => "janeiro", "02" => "fevereiro", "03" => "março",
        "04" => "abril",   "05" => "maio",       "06" => "junho",
        "07" => "julho",   "08" => "agosto",     "09" => "setembro",
        "10" => "outubro", "11" => "novembro",   "12" => "dezembro",
        _ => parts[1],
    };
    format!("{} de {} de {}", dia, mes, parts[0])
}

/// Normal paragraph with given text and half-point font size
fn par_sz(text: &str, sz: usize) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text).size(sz))
}

/// Bold paragraph with given text and half-point font size
fn par_bold_sz(text: &str, sz: usize) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text).bold().size(sz))
}

/// Empty paragraph for spacing
fn par_empty() -> Paragraph { Paragraph::new() }

/// Horizontal rule: an empty paragraph with a bottom border
fn par_hline() -> Paragraph {
    let mut p = Paragraph::new();
    p.property = p.property
        .set_border(ParagraphBorder::new(ParagraphBorderPosition::Bottom).size(6).color("000000"));
    p
}

/// Indented paragraph (left = 720 twips ≈ 12.7mm)
fn par_indent(text: &str, sz: usize) -> Paragraph {
    Paragraph::new()
        .indent(Some(720), None, None, None)
        .add_run(Run::new().add_text(text).size(sz))
}

// ── main command ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_prova_word(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, rodape, nome_escola, cidade, diretor, professor, data, logo_path):
        (String, String, String, String, String, String, String, String, String) = conn.query_row(
        "SELECT p.titulo, p.descricao, p.rodape,
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''), COALESCE(c.diretor,''),
                COALESCE(m.professor,''), p.data, COALESCE(c.logo_path,'')
         FROM provas p
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=p.materia_id
         WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta \
         FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut doc = Docx::new();

    // ── LOGO ─────────────────────────────────────────────────────────────────
    if !logo_path.is_empty() {
        if let Ok(dyn_img) = ic::open(&logo_path) {
            let (iw, ih) = (dyn_img.width(), dyn_img.height());
            let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
            if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                let png_bytes = png_buf.into_inner();
                // Target: 25 mm wide = 25 * 36000 EMU
                let target_w = 25_u32 * 36_000;
                let target_h = (target_w as f64 * ih as f64 / iw as f64) as u32;
                let mut pic = Pic::new_with_dimensions(png_bytes, iw, ih);
                pic.size = (target_w, target_h);
                doc = doc.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_image(pic))
                );
            }
        }
    }

    // ── INSTITUTION BLOCK ────────────────────────────────────────────────────
    if !nome_escola.is_empty() {
        doc = doc.add_paragraph(par_bold_sz(&nome_escola, 28));
    }
    let data_cidade = match (!cidade.is_empty(), !data.is_empty()) {
        (true,  true)  => format!("{}, {}", cidade, format_date_pt(&data)),
        (true,  false) => cidade.clone(),
        (false, true)  => format_date_pt(&data),
        (false, false) => String::new(),
    };
    if !data_cidade.is_empty() { doc = doc.add_paragraph(par_sz(&data_cidade, 20)); }
    if !diretor.is_empty()     { doc = doc.add_paragraph(par_sz(&format!("Diretor(a): {}", diretor), 20)); }
    if !professor.is_empty()   { doc = doc.add_paragraph(par_sz(&format!("Professor(a): {}", professor), 20)); }

    doc = doc.add_paragraph(par_hline());

    // ── STUDENT FIELDS ───────────────────────────────────────────────────────
    doc = doc.add_paragraph(par_sz(
        "Aluno(a): _____________________________________________", 22,
    ));
    doc = doc.add_paragraph(par_sz(
        "Série/Ano: ___________  Turno: ___________  Valor: _________  Nota: _________", 21,
    ));

    doc = doc.add_paragraph(par_hline());
    doc = doc.add_paragraph(par_empty());

    // ── EXAM TITLE ───────────────────────────────────────────────────────────
    doc = doc.add_paragraph(
        par_bold_sz(&titulo, 30).align(AlignmentType::Center),
    );
    if !descricao.is_empty() {
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&descricao).italic().size(20))
                .align(AlignmentType::Center),
        );
    }
    doc = doc.add_paragraph(par_empty());
    doc = doc.add_paragraph(par_hline());
    doc = doc.add_paragraph(par_empty());

    // ── QUESTIONS ────────────────────────────────────────────────────────────
    for (i, q) in questoes.iter().enumerate() {
        let opcoes_arr = q.opcoes.as_array();

        // Question header: bold, number + value
        doc = doc.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(&format!("Questão {}   ({:.1} pt)", i + 1, q.valor))
                    .bold()
                    .size(24),
            ),
        );

        // Enunciado (rich HTML → docx blocks)
        for block in html_to_blocks(&q.enunciado) {
            doc = doc.add_paragraph(block_to_paragraph(&block, Some(360)));
        }

        // Options / answer area
        match q.tipo.as_str() {
            "multipla_escolha" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            let letra = (b'a' + j as u8) as char;
                            doc = doc.add_paragraph(par_indent(&format!("{}) {}", letra, texto), 21));
                        }
                    }
                }
            }
            "verdadeiro_falso" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            let letra = (b'a' + j as u8) as char;
                            doc = doc.add_paragraph(par_indent(
                                &format!("{}) (  ) V    (  ) F    {}", letra, texto), 21,
                            ));
                        }
                    }
                }
            }
            "completar_lacunas" => {
                if let Some(opcoes) = opcoes_arr {
                    let palavras: Vec<&str> = opcoes.iter()
                        .filter_map(|o| o.get("texto").and_then(|t| t.as_str()))
                        .collect();
                    if !palavras.is_empty() {
                        doc = doc.add_paragraph(par_indent(
                            &format!("Banco de palavras: {}", palavras.join("  |  ")), 21,
                        ));
                    }
                }
            }
            "associacao" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        let texto_a = o.get("texto").and_then(|t| t.as_str()).unwrap_or("");
                        let texto_b = o.get("par").and_then(|t| t.as_str()).unwrap_or("");
                        let letra   = (b'A' + j as u8) as char;
                        doc = doc.add_paragraph(par_indent(
                            &format!("{}) {:<35} (  ) {}) {}", j + 1, texto_a, letra, texto_b), 21,
                        ));
                    }
                }
            }
            "ordenar" => {
                if let Some(opcoes) = opcoes_arr {
                    for o in opcoes.iter() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            doc = doc.add_paragraph(par_indent(&format!("(   ) {}", texto), 21));
                        }
                    }
                }
            }
            _ => {
                for _ in 0..q.linhas_resposta {
                    doc = doc.add_paragraph(par_indent("___________________________________________", 21));
                }
            }
        }
        doc = doc.add_paragraph(par_empty());
    }

    if !rodape.is_empty() {
        doc = doc.add_paragraph(par_hline());
        doc = doc.add_paragraph(par_sz(&rodape, 18));
    }

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| e.to_string())?;
    Ok(())
}
