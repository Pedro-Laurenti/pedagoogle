use docx_rs::*;
use printpdf::image_crate as ic;
use crate::db::get_conn;
use crate::models::Questao;
use crate::html_render::{html_to_blocks, Block, Align, TableRowData};
use crate::typst_pdf::render_latex_to_png;
use rusqlite::params;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Convert a parsed HTML Block into a docx Paragraph (for non-table blocks)
/// Math spans are rendered as images using Typst.
fn block_to_paragraph(block: &Block, indent_twips: Option<i32>) -> Option<Paragraph> {
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
                if span.is_math {
                    // Render LaTeX as image using Typst
                    if let Some(latex) = &span.latex {
                        if let Ok((png_data, width, height)) = render_latex_to_png(latex) {
                            // Convert pixels to EMUs (1 inch = 914400 EMUs, 96 DPI)
                            // Since we render at 2x scale, divide dimensions by 2
                            let emu_per_px = 914400 / 96 / 2;
                            let width_emu = width * emu_per_px;
                            let height_emu = height * emu_per_px;
                            
                            let pic = Pic::new_with_dimensions(png_data, width, height)
                                .size(width_emu, height_emu);
                            p = p.add_run(Run::new().add_image(pic));
                            continue;
                        }
                    }
                    // Fallback to text if rendering fails
                    let run = Run::new().add_text(&span.text).size(sz).italic();
                    p = p.add_run(run);
                } else {
                    if span.text.is_empty() { continue; }
                    let mut run = Run::new().add_text(&span.text).size(sz);
                    if span.bold    { run = run.bold(); }
                    if span.italic  { run = run.italic(); }
                    if span.underline { run = run.underline("single"); }
                    p = p.add_run(run);
                }
            }
            Some(p)
        }
        Block::Hr => Some(par_hline()),
        Block::Table { .. } => None, // Tables handled separately
        Block::Image { .. } => None, // Images handled separately
    }
}

/// Create a docx Table from table data
fn create_docx_table(rows: &[TableRowData]) -> Table {
    let mut table_rows = Vec::new();
    
    for row_data in rows {
        let cells: Vec<TableCell> = row_data.cells.iter().map(|cell_text| {
            let mut run = Run::new().add_text(cell_text).size(20);
            if row_data.is_header {
                run = run.bold();
            }
            let para = Paragraph::new().add_run(run);
            TableCell::new().add_paragraph(para)
        }).collect();
        table_rows.push(TableRow::new(cells));
    }
    
    Table::new(table_rows)
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

/// Horizontal rule: an empty paragraph with a bottom border only.
/// NOTE: set_border() uses unwrap_or_default() which adds all 4 borders (box).
/// We must use set_borders() with ParagraphBorders::with_empty() to get only one side.
fn par_hline() -> Paragraph {
    let mut p = Paragraph::new();
    p.property = p.property.set_borders(
        ParagraphBorders::with_empty()
            .set(ParagraphBorder::new(ParagraphBorderPosition::Bottom).size(6).color("000000")),
    );
    p
}

/// Indented paragraph (left = 720 twips ≈ 12.7mm)
fn par_indent(text: &str, sz: usize) -> Paragraph {
    Paragraph::new()
        .indent(Some(720), None, None, None)
        .add_run(Run::new().add_text(text).size(sz))
}

// ── Word page-border XML injection ─────────────────────────────────────────

/// Inject `<w:pgBorders>` into the sectPr element of the built document XML.
/// `space` is distance from page edge in points (1 pt ≈ 0.353 mm).
fn inject_word_page_border(document_xml: Vec<u8>, estilo: &str, margem_folha: f64) -> Vec<u8> {
    let (val, sz) = match estilo {
        "simple"  => ("single", "4"),
        "double"  => ("double", "6"),
        "ornate"  => ("wavyDouble", "6"),
        "classic" => ("thick",  "12"),
        "modern"  => ("triple", "8"),
        _         => return document_xml,
    };
    let space = (margem_folha * 2.8346) as u32; // mm → pt
    let borders = format!(
        r#"<w:pgBorders w:offsetFrom="page">\
<w:top w:val="{v}" w:sz="{sz}" w:space="{sp}" w:color="000000"/>\
<w:left w:val="{v}" w:sz="{sz}" w:space="{sp}" w:color="000000"/>\
<w:bottom w:val="{v}" w:sz="{sz}" w:space="{sp}" w:color="000000"/>\
<w:right w:val="{v}" w:sz="{sz}" w:space="{sp}" w:color="000000"/>\
</w:pgBorders>"#,
        v = val, sz = sz, sp = space
    );
    let xml = String::from_utf8_lossy(&document_xml);
    if let Some(pos) = xml.find("</w:sectPr>") {
        let mut out = document_xml[..pos].to_vec();
        out.extend_from_slice(borders.as_bytes());
        out.extend_from_slice(&document_xml[pos..]);
        out
    } else {
        document_xml
    }
}

// ── main command ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_prova_word(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, rodape, nome_escola, cidade, diretor, professor, data, logo_path,
         moldura_estilo, margem_folha, margem_moldura, margem_conteudo, prova_margens):
        (String, String, String, String, String, String, String, String, String, String, f64, f64, f64, String) = conn.query_row(
        "SELECT p.titulo, p.descricao, p.rodape,
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''), COALESCE(c.diretor,''),
                COALESCE(m.professor,''), p.data, COALESCE(c.logo_path,''),
                COALESCE(c.moldura_estilo,'none'),
                COALESCE(c.margem_folha, 10.0), COALESCE(c.margem_moldura, 5.0), COALESCE(c.margem_conteudo, 5.0),
                p.margens
         FROM provas p
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=p.materia_id
         WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?)),
    ).map_err(|e| e.to_string())?;
    let margem_folha = match prova_margens.as_str() {
        "estreito" => 15.0,
        "normal"   => 20.0,
        "largo"    => 25.0,
        _          => margem_folha,
    };

    // Calculate total margin = paper margin + frame margin + content margin
    // Convert mm to twips: 1 mm ≈ 56.7 twips (1 inch = 1440 twips, 1 inch = 25.4 mm)
    let total_margin_mm = margem_folha + margem_moldura + margem_conteudo;
    let margin_twips = (total_margin_mm * 56.7) as i32;

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

    // Create document with custom page margins
    let page_margin = PageMargin::new()
        .top(margin_twips)
        .bottom(margin_twips)
        .left(margin_twips)
        .right(margin_twips);
    let mut doc = Docx::new().page_margin(page_margin);

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
            match &block {
                Block::Table { rows } => {
                    doc = doc.add_table(create_docx_table(rows));
                }
                Block::Image { data, width, height: _ } => {
                    // Decode image dimensions from PNG/JPEG data
                    if let Ok(dyn_img) = ic::load_from_memory(data) {
                        let (iw, ih) = (dyn_img.width(), dyn_img.height());
                        
                        // Convert to PNG for DOCX
                        let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
                        if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                            let png_bytes = png_buf.into_inner();
                            
                            // Calculate EMUs: if width specified, use it; otherwise use image width at 96 DPI
                            // 1 inch = 914400 EMUs, 1 inch = 96 pixels at 96 DPI
                            let emu_per_px = 914400 / 96;
                            let (target_w, target_h) = if let Some(w) = width {
                                let tw = (*w as u32) * emu_per_px;
                                let th = (tw as f64 * ih as f64 / iw as f64) as u32;
                                (tw, th)
                            } else {
                                // Limit max width to ~15cm (about 567px at 96DPI)
                                let max_width = 567u32;
                                let actual_w = iw.min(max_width);
                                let tw = actual_w * emu_per_px;
                                let th = (tw as f64 * ih as f64 / iw as f64) as u32;
                                (tw, th)
                            };
                            
                            let pic = Pic::new_with_dimensions(png_bytes, iw, ih)
                                .size(target_w, target_h);
                            doc = doc.add_paragraph(
                                Paragraph::new()
                                    .add_run(Run::new().add_image(pic))
                                    .indent(Some(360), None, None, None)
                            );
                        }
                    }
                }
                _ => {
                    if let Some(p) = block_to_paragraph(&block, Some(360)) {
                        doc = doc.add_paragraph(p);
                    }
                }
            }
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
    let mut xml_docx = doc.build();
    if moldura_estilo != "none" {
        xml_docx.document = inject_word_page_border(
            xml_docx.document, &moldura_estilo, margem_folha
        );
    }
    xml_docx.pack(file).map_err(|e| e.to_string())?;
    Ok(())
}
