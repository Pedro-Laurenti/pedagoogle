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

/// A full-width answer line using a bottom border paragraph (fills available column width).
fn par_answer_line() -> Paragraph {
    let mut p = Paragraph::new();
    p.property = p.property.set_borders(
        ParagraphBorders::with_empty()
            .set(ParagraphBorder::new(ParagraphBorderPosition::Bottom).size(4).color("AAAAAA")),
    );
    p
}

/// A full-width draft/rascunho line — lighter border to differentiate from answer lines.
fn par_draft_line() -> Paragraph {
    let mut p = Paragraph::new();
    p.property = p.property.set_borders(
        ParagraphBorders::with_empty()
            .set(ParagraphBorder::new(ParagraphBorderPosition::Bottom).size(2).color("CCCCCC")),
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

    let (titulo, descricao, nome_escola, cidade, estado, diretor, professor, logo_path,
         moldura_estilo, margem_folha, margem_moldura, margem_conteudo):
        (String, String, String, String, String, String, String, String, String, f64, f64, f64) = conn.query_row(
        "SELECT p.titulo, p.descricao,
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''), COALESCE(c.estado,''),
                COALESCE(c.diretor,''), COALESCE(prof.nome,''), COALESCE(c.logo_path,''),
                COALESCE(c.moldura_estilo,'none'),
                COALESCE(c.margem_folha, 10.0), COALESCE(c.margem_moldura, 5.0), COALESCE(c.margem_conteudo, 5.0)
         FROM provas p
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=p.materia_id
         LEFT JOIN professores prof ON prof.id=m.professor_id
         WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?)),
    ).map_err(|e| e.to_string())?;

    // Calculate total margin = paper margin + frame margin + content margin
    // Convert mm to twips: 1 mm ≈ 56.7 twips (1 inch = 1440 twips, 1 inch = 25.4 mm)
    let total_margin_mm = margem_folha + margem_moldura + margem_conteudo;
    let margin_twips = (total_margin_mm * 56.7) as i32;
    // 60% of text column width in EMU (A4 = 210mm, 1mm = 36000 EMU)
    let col_60_emu = ((210.0 - 2.0 * total_margin_mm) * 36000.0 * 0.6) as u32;

    let mut stmt = conn.prepare(
        "SELECT id, prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, \
         COALESCE(resposta,''), COALESCE(espaco_rascunho,0) \
         FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
            resposta: r.get(8)?, espaco_rascunho: r.get(9)?,
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

    // ── HEADER (unified bordered block) ────────────────────────────────────
    let cidade_estado = if !estado.is_empty() && !cidade.is_empty() {
        format!("{} \u{2014} {}", cidade, estado)
    } else if !estado.is_empty() {
        estado.clone()
    } else {
        cidade.clone()
    };

    // Build field paragraphs for the right column
    let mut header_cell = TableCell::new();

    // Line 1: School name (plain, no bold, no centering)
    if !nome_escola.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&nome_escola.to_uppercase(), 20));
    }
    // Line 2: Diretor
    if !diretor.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&format!("Diretor(a): {}", diretor), 20));
    } else {
        header_cell = header_cell.add_paragraph(par_sz("Diretor(a): ___________________________________________", 20));
    }
    // Line 3: Cidade, Estado, Data
    {
        let loc_part = if !cidade_estado.is_empty() {
            format!("{}, ", cidade_estado)
        } else { String::new() };
        header_cell = header_cell.add_paragraph(par_sz(&format!("{}Data: ____/____/________", loc_part), 20));
    }
    // Line 4: Professor
    if !professor.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&format!("Professor(a): {}", professor), 20));
    } else {
        header_cell = header_cell.add_paragraph(par_sz("Professor(a): ___________________________________________", 20));
    }
    // Line 5: Aluno
    header_cell = header_cell.add_paragraph(par_sz("Aluno(a): _______________________________________________", 20));
    // Line 6: Ano | Turma | Turno
    header_cell = header_cell.add_paragraph(par_sz("Ano: ________   Turma: ________   Turno: ________", 20));
    // Line 7: Valor | Nota
    header_cell = header_cell.add_paragraph(par_sz("Valor: ________   Nota: ________", 20));

    // Thin gray border for the whole header table
    let hdr_border = TableBorders::new()
        .set(TableBorder::new(TableBorderPosition::Top).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Bottom).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Left).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Right).size(4).color("B0B0B0").border_type(BorderType::Single))
        .clear(TableBorderPosition::InsideH)
        .clear(TableBorderPosition::InsideV);

    // A4 content width in twips = (210mm - 2*margin) * 56.7
    let content_width_twips = ((210.0 - 2.0 * total_margin_mm) * 56.7) as usize;

    if !logo_path.is_empty() {
        if let Ok(dyn_img) = ic::open(&logo_path) {
            let (iw, ih) = (dyn_img.width(), dyn_img.height());
            let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
            if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                let png_bytes = png_buf.into_inner();
                // Logo cell = 1/10 of content width, spacer = ~5mm gap
                let logo_cell_twips = content_width_twips * 1 / 10;
                let spacer_twips = 280usize;
                let fields_cell_twips = content_width_twips - logo_cell_twips - spacer_twips;
                let logo_emu_w = (logo_cell_twips as u32) * 635; // twips to EMU: 1 twip = 635 EMU (approx)
                let logo_emu_h = (logo_emu_w as f64 * ih as f64 / iw as f64) as u32;
                let mut pic = Pic::new_with_dimensions(png_bytes, iw, ih);
                pic.size = (logo_emu_w, logo_emu_h);
                let logo_cell = TableCell::new()
                    .add_paragraph(Paragraph::new().align(AlignmentType::Center).add_run(Run::new().add_image(pic)))
                    .vertical_align(VAlignType::Center)
                    .width(logo_cell_twips, WidthType::Dxa)
                    .clear_all_border();
                let spacer_cell = TableCell::new()
                    .width(spacer_twips, WidthType::Dxa)
                    .clear_all_border();
                let header_cell = header_cell
                    .width(fields_cell_twips, WidthType::Dxa)
                    .clear_all_border();
                let header_table = Table::new(vec![TableRow::new(vec![logo_cell, spacer_cell, header_cell])])
                    .set_grid(vec![logo_cell_twips, spacer_twips, fields_cell_twips])
                    .width(content_width_twips, WidthType::Dxa)
                    .set_borders(hdr_border);
                doc = doc.add_table(header_table);
            } else {
                // Fallback: no logo, single cell
                let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
                let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
                    .set_grid(vec![content_width_twips])
                    .width(content_width_twips, WidthType::Dxa)
                    .set_borders(hdr_border);
                doc = doc.add_table(header_table);
            }
        } else {
            // Logo file not found, fallback
            let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
            let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
                .set_grid(vec![content_width_twips])
                .width(content_width_twips, WidthType::Dxa)
                .set_borders(hdr_border);
            doc = doc.add_table(header_table);
        }
    } else {
        // No logo path
        let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
        let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
            .set_grid(vec![content_width_twips])
            .width(content_width_twips, WidthType::Dxa)
            .set_borders(hdr_border);
        doc = doc.add_table(header_table);
    }

    doc = doc.add_paragraph(par_empty());

    // ── EXAM TITLE ───────────────────────────────────────────────────────────
    doc = doc.add_paragraph(
        par_bold_sz(&titulo, 24).align(AlignmentType::Center),
    );
    if !descricao.is_empty() {
        for block in html_to_blocks(&descricao) {
            match &block {
                Block::Table { rows } => { doc = doc.add_table(create_docx_table(rows)); }
                _ => {
                    if let Some(p) = block_to_paragraph(&block, None) {
                        doc = doc.add_paragraph(p.align(AlignmentType::Center));
                    }
                }
            }
        }
    }
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
                            
                            // Calculate EMUs: cap at 60% of column width
                            let emu_per_px = 914400 / 96;
                            let native_w = if let Some(w) = width {
                                (*w as u32) * emu_per_px
                            } else {
                                iw * emu_per_px
                            };
                            let target_w = native_w.min(col_60_emu);
                            let target_h = (target_w as f64 * ih as f64 / iw as f64) as u32;
                            
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
            "letras" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        let letra = (b'A' + j as u8) as char;
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            doc = doc.add_paragraph(par_indent(
                                &format!("{})\t{}", letra, texto), 21,
                            ));
                        }
                        let linhas_sub = o.get("linhas").and_then(|v| v.as_i64()).unwrap_or(1);
                        for _ in 0..linhas_sub {
                            doc = doc.add_paragraph(par_answer_line());
                        }
                    }
                }
            }
            _ => {}
        }
        for _ in 0..q.linhas_resposta {
            doc = doc.add_paragraph(par_answer_line());
        }
        for _ in 0..q.espaco_rascunho {
            doc = doc.add_paragraph(par_draft_line());
        }
        doc = doc.add_paragraph(par_empty());
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

#[tauri::command]
pub fn export_atividade_word(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, nome_materia, nome_escola, cidade, estado, diretor, professor, logo_path,
         moldura_estilo, margem_folha, margem_moldura, margem_conteudo, vale_nota, valor_total):
        (String, String, String, String, String, String, String, String, String, String, f64, f64, f64, i64, f64) = conn.query_row(
        "SELECT a.titulo, a.descricao, COALESCE(m.nome,''),
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''), COALESCE(c.estado,''),
                COALESCE(c.diretor,''), COALESCE(prof.nome,''), COALESCE(c.logo_path,''),
                COALESCE(c.moldura_estilo,'none'),
                COALESCE(c.margem_folha, 10.0), COALESCE(c.margem_moldura, 5.0), COALESCE(c.margem_conteudo, 5.0),
                a.vale_nota, a.valor_total
         FROM atividades a
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=a.materia_id
         LEFT JOIN professores prof ON prof.id=m.professor_id
         WHERE a.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?, r.get(14)?)),
    ).map_err(|e| e.to_string())?;

    let total_margin_mm = margem_folha + margem_moldura + margem_conteudo;
    let margin_twips = (total_margin_mm * 56.7) as i32;
    let col_60_emu = ((210.0 - 2.0 * total_margin_mm) * 36000.0 * 0.6) as u32;

    let mut stmt = conn.prepare(
        "SELECT id, atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, \
         COALESCE(resposta,''), COALESCE(espaco_rascunho,0) \
         FROM questoes_atividade WHERE atividade_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
            resposta: r.get(8)?, espaco_rascunho: r.get(9)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let page_margin = PageMargin::new()
        .top(margin_twips).bottom(margin_twips)
        .left(margin_twips).right(margin_twips);
    let mut doc = Docx::new().page_margin(page_margin);

    // ── HEADER (unified bordered block) ────────────────────────────────────
    let cidade_estado = if !estado.is_empty() && !cidade.is_empty() {
        format!("{} \u{2014} {}", cidade, estado)
    } else if !estado.is_empty() { estado.clone() } else { cidade.clone() };

    let mut header_cell = TableCell::new();

    // Line 1: School name (plain, no bold, no centering)
    if !nome_escola.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&nome_escola.to_uppercase(), 20));
    }
    // Line 2: Diretor
    if !diretor.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&format!("Diretor(a): {}", diretor), 20));
    } else {
        header_cell = header_cell.add_paragraph(par_sz("Diretor(a): ___________________________________________", 20));
    }
    // Line 3: Cidade, Estado, Data
    {
        let loc_part = if !cidade_estado.is_empty() {
            format!("{}, ", cidade_estado)
        } else { String::new() };
        header_cell = header_cell.add_paragraph(par_sz(&format!("{}Data: ____/____/________", loc_part), 20));
    }
    // Line 4: Professor
    if !professor.is_empty() {
        header_cell = header_cell.add_paragraph(par_sz(&format!("Professor(a): {}", professor), 20));
    } else {
        header_cell = header_cell.add_paragraph(par_sz("Professor(a): ___________________________________________", 20));
    }
    // Line 5: Aluno
    header_cell = header_cell.add_paragraph(par_sz("Aluno(a): _______________________________________________", 20));
    // Line 6: Ano | Turma | Turno
    header_cell = header_cell.add_paragraph(par_sz("Ano: ________   Turma: ________   Turno: ________", 20));
    // Line 7: Valor | Nota
    if vale_nota != 0 {
        header_cell = header_cell.add_paragraph(par_sz(&format!("Valor: {:.1}   Nota: ________", valor_total), 20));
    } else {
        header_cell = header_cell.add_paragraph(par_sz("Valor: ________   Nota: ________", 20));
    }

    let hdr_border = TableBorders::new()
        .set(TableBorder::new(TableBorderPosition::Top).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Bottom).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Left).size(4).color("B0B0B0").border_type(BorderType::Single))
        .set(TableBorder::new(TableBorderPosition::Right).size(4).color("B0B0B0").border_type(BorderType::Single))
        .clear(TableBorderPosition::InsideH)
        .clear(TableBorderPosition::InsideV);

    let content_width_twips = ((210.0 - 2.0 * total_margin_mm) * 56.7) as usize;

    if !logo_path.is_empty() {
        if let Ok(dyn_img) = ic::open(&logo_path) {
            let (iw, ih) = (dyn_img.width(), dyn_img.height());
            let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
            if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                let png_bytes = png_buf.into_inner();
                // Logo cell = 1/10 of content width, spacer = ~5mm gap
                let logo_cell_twips = content_width_twips * 1 / 10;
                let spacer_twips = 280usize;
                let fields_cell_twips = content_width_twips - logo_cell_twips - spacer_twips;
                let logo_emu_w = (logo_cell_twips as u32) * 635;
                let logo_emu_h = (logo_emu_w as f64 * ih as f64 / iw as f64) as u32;
                let mut pic = Pic::new_with_dimensions(png_bytes, iw, ih);
                pic.size = (logo_emu_w, logo_emu_h);
                let logo_cell = TableCell::new()
                    .add_paragraph(Paragraph::new().align(AlignmentType::Center).add_run(Run::new().add_image(pic)))
                    .vertical_align(VAlignType::Center)
                    .width(logo_cell_twips, WidthType::Dxa)
                    .clear_all_border();
                let spacer_cell = TableCell::new()
                    .width(spacer_twips, WidthType::Dxa)
                    .clear_all_border();
                let header_cell = header_cell
                    .width(fields_cell_twips, WidthType::Dxa)
                    .clear_all_border();
                let header_table = Table::new(vec![TableRow::new(vec![logo_cell, spacer_cell, header_cell])])
                    .set_grid(vec![logo_cell_twips, spacer_twips, fields_cell_twips])
                    .width(content_width_twips, WidthType::Dxa)
                    .set_borders(hdr_border);
                doc = doc.add_table(header_table);
            } else {
                let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
                let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
                    .set_grid(vec![content_width_twips])
                    .width(content_width_twips, WidthType::Dxa)
                    .set_borders(hdr_border);
                doc = doc.add_table(header_table);
            }
        } else {
            let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
            let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
                .set_grid(vec![content_width_twips])
                .width(content_width_twips, WidthType::Dxa)
                .set_borders(hdr_border);
            doc = doc.add_table(header_table);
        }
    } else {
        let header_cell = header_cell.width(content_width_twips, WidthType::Dxa).clear_all_border();
        let header_table = Table::new(vec![TableRow::new(vec![header_cell])])
            .set_grid(vec![content_width_twips])
            .width(content_width_twips, WidthType::Dxa)
            .set_borders(hdr_border);
        doc = doc.add_table(header_table);
    }

    doc = doc.add_paragraph(par_empty());

    // ── ACTIVITY TITLE ───────────────────────────────────────────────────────
    if !nome_materia.is_empty() {
        doc = doc.add_paragraph(par_sz(&nome_materia, 20).align(AlignmentType::Center));
    }
    doc = doc.add_paragraph(par_bold_sz(&titulo, 24).align(AlignmentType::Center));
    if !descricao.is_empty() {
        for block in html_to_blocks(&descricao) {
            match &block {
                Block::Table { rows } => { doc = doc.add_table(create_docx_table(rows)); }
                _ => {
                    if let Some(p) = block_to_paragraph(&block, None) {
                        doc = doc.add_paragraph(p.align(AlignmentType::Center));
                    }
                }
            }
        }
    }
    doc = doc.add_paragraph(par_empty());

    let mut questao_num = 0usize;
    for q in questoes.iter() {
        let opcoes_arr = q.opcoes.as_array();
        if q.tipo == "texto" {
            for block in html_to_blocks(&q.enunciado) {
                match &block {
                    Block::Table { rows } => { doc = doc.add_table(create_docx_table(rows)); }
                    Block::Image { data, width, height: _ } => {
                        if let Ok(dyn_img) = ic::load_from_memory(data) {
                            let (iw, ih) = (dyn_img.width(), dyn_img.height());
                            let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
                            if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                                let png_bytes = png_buf.into_inner();
                                let emu_per_px = 914400 / 96;
                                let native_w = width.map(|w| w as u32 * emu_per_px).unwrap_or(iw * emu_per_px);
                                let target_w = native_w.min(col_60_emu);
                                let target_h = (target_w as f64 * ih as f64 / iw as f64) as u32;
                                let pic = Pic::new_with_dimensions(png_bytes, iw, ih).size(target_w, target_h);
                                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic)).indent(Some(360), None, None, None));
                            }
                        }
                    }
                    _ => { if let Some(p) = block_to_paragraph(&block, Some(360)) { doc = doc.add_paragraph(p); } }
                }
            }
            continue;
        }
        questao_num += 1;
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("Questão {}   ({:.1} pt)", questao_num, q.valor)).bold().size(24)));

        for block in html_to_blocks(&q.enunciado) {
            match &block {
                Block::Table { rows } => { doc = doc.add_table(create_docx_table(rows)); }
                Block::Image { data, width, height: _ } => {
                    if let Ok(dyn_img) = ic::load_from_memory(data) {
                        let (iw, ih) = (dyn_img.width(), dyn_img.height());
                        let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
                        if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                            let png_bytes = png_buf.into_inner();
                            let emu_per_px = 914400 / 96;
                            let native_w = width.map(|w| w as u32 * emu_per_px).unwrap_or(iw * emu_per_px);
                            let target_w = native_w.min(col_60_emu);
                            let target_h = (target_w as f64 * ih as f64 / iw as f64) as u32;
                            let pic = Pic::new_with_dimensions(png_bytes, iw, ih).size(target_w, target_h);
                            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic)).indent(Some(360), None, None, None));
                        }
                    }
                }
                _ => { if let Some(p) = block_to_paragraph(&block, Some(360)) { doc = doc.add_paragraph(p); } }
            }
        }

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
                            doc = doc.add_paragraph(par_indent(&format!("{}) (  ) V    (  ) F    {}", letra, texto), 21));
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
                        doc = doc.add_paragraph(par_indent(&format!("Banco de palavras: {}", palavras.join("  |  ")), 21));
                    }
                }
            }
            "associacao" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        let texto_a = o.get("texto").and_then(|t| t.as_str()).unwrap_or("");
                        let texto_b = o.get("par").and_then(|t| t.as_str()).unwrap_or("");
                        let letra = (b'A' + j as u8) as char;
                        doc = doc.add_paragraph(par_indent(&format!("{}) {:<35} (  ) {}) {}", j + 1, texto_a, letra, texto_b), 21));
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
            "letras" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        let letra = (b'A' + j as u8) as char;
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            doc = doc.add_paragraph(par_indent(&format!("{})\t{}", letra, texto), 21));
                        }
                        let linhas_sub = o.get("linhas").and_then(|v| v.as_i64()).unwrap_or(1);
                        for _ in 0..linhas_sub { doc = doc.add_paragraph(par_answer_line()); }
                    }
                }
            }
            _ => {}
        }
        for _ in 0..q.linhas_resposta { doc = doc.add_paragraph(par_answer_line()); }
        for _ in 0..q.espaco_rascunho { doc = doc.add_paragraph(par_draft_line()); }
        doc = doc.add_paragraph(par_empty());
    }

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut xml_docx = doc.build();
    if moldura_estilo != "none" {
        xml_docx.document = inject_word_page_border(xml_docx.document, &moldura_estilo, margem_folha);
    }
    xml_docx.pack(file).map_err(|e| e.to_string())?;
    Ok(())
}

// ── Gabarito Word helpers & commands ─────────────────────────────────────────

fn append_gabarito_question(mut doc: Docx, q: &Questao, questao_num: usize, col_emu: u32) -> Docx {
    // Question header
    doc = doc.add_paragraph(
        Paragraph::new().add_run(
            Run::new()
                .add_text(&format!("Questão {}   ({:.1} pt)", questao_num, q.valor))
                .bold()
                .size(24),
        ),
    );

    // Enunciado
    for block in html_to_blocks(&q.enunciado) {
        match &block {
            Block::Table { rows } => { doc = doc.add_table(create_docx_table(rows)); }
            Block::Image { data, width, height: _ } => {
                if let Ok(dyn_img) = ic::load_from_memory(data) {
                    let (iw, ih) = (dyn_img.width(), dyn_img.height());
                    let mut png_buf = std::io::Cursor::new(Vec::<u8>::new());
                    if dyn_img.write_to(&mut png_buf, ic::ImageFormat::Png).is_ok() {
                        let png_bytes = png_buf.into_inner();
                        let emu_per_px = 914400 / 96;
                        let native_w = if let Some(w) = width { (*w as u32) * emu_per_px } else { iw * emu_per_px };
                        let target_w = native_w.min(col_emu);
                        let target_h = (target_w as f64 * ih as f64 / iw as f64) as u32;
                        let pic = Pic::new_with_dimensions(png_bytes, iw, ih).size(target_w, target_h);
                        doc = doc.add_paragraph(
                            Paragraph::new().add_run(Run::new().add_image(pic)).indent(Some(360), None, None, None)
                        );
                    }
                }
            }
            _ => {
                if let Some(p) = block_to_paragraph(&block, Some(360)) { doc = doc.add_paragraph(p); }
            }
        }
    }

    let opcoes_arr = q.opcoes.as_array();
    match q.tipo.as_str() {
        "multipla_escolha" => {
            if let Some(opcoes) = opcoes_arr {
                for (j, o) in opcoes.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        let letra = (b'a' + j as u8) as char;
                        let correto = o.get("correta").and_then(|v| v.as_bool()).unwrap_or(false);
                        if correto {
                            let run = Run::new()
                                .add_text(&format!("{}) {} [CORRETA]", letra, texto))
                                .size(21).bold();
                            doc = doc.add_paragraph(
                                Paragraph::new().indent(Some(720), None, None, None).add_run(run)
                            );
                        } else {
                            doc = doc.add_paragraph(par_indent(&format!("{}) {}", letra, texto), 21));
                        }
                    }
                }
            }
        }
        "verdadeiro_falso" => {
            if let Some(opcoes) = opcoes_arr {
                for (j, o) in opcoes.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        let letra = (b'a' + j as u8) as char;
                        let correta = o.get("correta").and_then(|v| v.as_bool()).unwrap_or(false);
                        let vf = if correta { "V" } else { "F" };
                        doc = doc.add_paragraph(
                            Paragraph::new()
                                .indent(Some(720), None, None, None)
                                .add_run(Run::new().add_text(&format!("{}) [{}]  ", letra, vf)).size(21).bold())
                                .add_run(Run::new().add_text(texto).size(21)),
                        );
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
            if !q.resposta.is_empty() {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .indent(Some(720), None, None, None)
                        .add_run(Run::new().add_text("Resposta: ").size(21).bold())
                        .add_run(Run::new().add_text(&q.resposta).size(21)),
                );
            }
        }
        "associacao" => {
            if let Some(opcoes) = opcoes_arr {
                for (j, o) in opcoes.iter().enumerate() {
                    let a = o.get("texto").and_then(|t| t.as_str()).unwrap_or("");
                    let b_text = o.get("par").and_then(|t| t.as_str()).unwrap_or("");
                    let letra = (b'A' + j as u8) as char;
                    doc = doc.add_paragraph(
                        Paragraph::new()
                            .indent(Some(720), None, None, None)
                            .add_run(Run::new().add_text(&format!("{})\u{00a0}{}  \u{2192}  ", j + 1, a)).size(21))
                            .add_run(Run::new().add_text(&format!("{}) {}", letra, b_text)).size(21).bold()),
                    );
                }
            }
        }
        "ordenar" => {
            if let Some(opcoes) = opcoes_arr {
                for (j, o) in opcoes.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        doc = doc.add_paragraph(
                            Paragraph::new()
                                .indent(Some(720), None, None, None)
                                .add_run(Run::new().add_text(&format!("({})  ", j + 1)).size(21).bold())
                                .add_run(Run::new().add_text(texto).size(21)),
                        );
                    }
                }
            }
        }
        "letras" => {
            if let Some(opcoes) = opcoes_arr {
                for (j, o) in opcoes.iter().enumerate() {
                    let letra = (b'A' + j as u8) as char;
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        doc = doc.add_paragraph(par_indent(&format!("{}) {}", letra, texto), 21));
                    }
                    let resp = o.get("par").and_then(|v| v.as_str()).unwrap_or("");
                    if !resp.is_empty() {
                        doc = doc.add_paragraph(
                            Paragraph::new()
                                .indent(Some(1080), None, None, None)
                                .add_run(Run::new().add_text("Resposta: ").size(21).bold())
                                .add_run(Run::new().add_text(resp).size(21)),
                        );
                    } else {
                        doc = doc.add_paragraph(
                            Paragraph::new()
                                .indent(Some(1080), None, None, None)
                                .add_run(Run::new().add_text("[Sem resposta cadastrada]").size(21).italic()),
                        );
                    }
                }
            }
        }
        _ => {
            if !q.resposta.is_empty() {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .indent(Some(720), None, None, None)
                        .add_run(Run::new().add_text("Resposta esperada: ").size(21).bold())
                        .add_run(Run::new().add_text(&q.resposta).size(21)),
                );
            } else {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .indent(Some(720), None, None, None)
                        .add_run(Run::new().add_text("[Sem resposta cadastrada]").size(21).italic()),
                );
            }
        }
    }
    doc = doc.add_paragraph(par_empty());
    doc
}

#[tauri::command]
pub fn export_gabarito_word(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let (titulo, nome_escola, professor): (String, String, String) = conn.query_row(
        "SELECT p.titulo, COALESCE(c.nome_escola,''), COALESCE(prof.nome,'') \
         FROM provas p LEFT JOIN configuracoes c ON c.id=1 \
         LEFT JOIN materias m ON m.id=p.materia_id \
         LEFT JOIN professores prof ON prof.id=m.professor_id \
         WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, prova_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, \
         COALESCE(resposta,''), COALESCE(espaco_rascunho,0) \
         FROM questoes WHERE prova_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
            resposta: r.get(8)?, espaco_rascunho: r.get(9)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 2cm margins = 20mm each side → 170mm usable → 60% for images
    let col_emu = (170.0_f64 * 36000.0 * 0.6) as u32;
    let margin_twips = (20.0_f64 * 56.693) as i32;
    let page_margin = PageMargin::new()
        .top(margin_twips).bottom(margin_twips).left(margin_twips).right(margin_twips);
    let mut doc = Docx::new().page_margin(page_margin);

    if !nome_escola.is_empty() {
        doc = doc.add_paragraph(par_bold_sz(&nome_escola, 28).align(AlignmentType::Center));
    }
    doc = doc.add_paragraph(
        par_bold_sz("GABARITO", 32).align(AlignmentType::Center)
    );
    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(&titulo).size(26))
    );
    if !professor.is_empty() {
        doc = doc.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(&format!("Prof.: {}", professor)).size(22))
        );
    }
    doc = doc.add_paragraph(par_hline());

    let mut questao_num = 0usize;
    for q in &questoes {
        if q.tipo == "texto" { continue; }
        questao_num += 1;
        doc = append_gabarito_question(doc, q, questao_num, col_emu);
    }

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn export_gabarito_atividade_word(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let (titulo, nome_escola): (String, String) = conn.query_row(
        "SELECT a.titulo, COALESCE(c.nome_escola,'') \
         FROM atividades a LEFT JOIN configuracoes c ON c.id=1 WHERE a.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta, \
         COALESCE(resposta,''), COALESCE(espaco_rascunho,0) \
         FROM questoes_atividade WHERE atividade_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<crate::models::Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(crate::models::Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
            resposta: r.get(8)?, espaco_rascunho: r.get(9)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let col_emu = (170.0_f64 * 36000.0 * 0.6) as u32;
    let margin_twips = (20.0_f64 * 56.693) as i32;
    let page_margin = PageMargin::new()
        .top(margin_twips).bottom(margin_twips).left(margin_twips).right(margin_twips);
    let mut doc = Docx::new().page_margin(page_margin);

    if !nome_escola.is_empty() {
        doc = doc.add_paragraph(par_bold_sz(&nome_escola, 28).align(AlignmentType::Center));
    }
    doc = doc.add_paragraph(
        par_bold_sz("GABARITO", 32).align(AlignmentType::Center)
    );
    doc = doc.add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(&titulo).size(26))
    );
    doc = doc.add_paragraph(par_hline());

    let mut questao_num = 0usize;
    for q in &questoes {
        if q.tipo == "texto" { continue; }
        questao_num += 1;
        doc = append_gabarito_question(doc, q, questao_num, col_emu);
    }

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    doc.build().pack(file).map_err(|e| e.to_string())?;
    Ok(())
}
