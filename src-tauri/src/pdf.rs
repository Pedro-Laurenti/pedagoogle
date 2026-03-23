use printpdf::*;
use printpdf::image_crate as ic;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use crate::db::get_conn;
use crate::models::Questao;
use crate::html_render::html_to_plain;
use rusqlite::params;

// Embed fonts at compile time so they work on every platform (Windows, macOS, Linux).
// LiberationSans has full Unicode coverage for math symbols (≠, π, α, ≤, etc.)
const FONT_REGULAR_BYTES: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");
const FONT_BOLD_BYTES:    &[u8] = include_bytes!("../fonts/LiberationSans-Bold.ttf");

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

fn draw_hline(layer: &PdfLayerReference, y: f32, x1: f32, x2: f32) {
    let line = Line::from_iter(vec![
        (Point::new(Mm(x1), Mm(y)), false),
        (Point::new(Mm(x2), Mm(y)), false),
    ]);
    layer.add_line(line);
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() { lines.push(current); }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

#[tauri::command]
pub fn export_prova_pdf(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, rodape, nome_escola, cidade, diretor, professor, data, logo_path):
        (String, String, String, String, String, String, String, String, String) = conn.query_row(
        "SELECT p.titulo, p.descricao, p.rodape,
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''), COALESCE(c.diretor,''),
                COALESCE(prof.nome,''), p.data, COALESCE(c.logo_path,'')
         FROM provas p
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=p.materia_id
         LEFT JOIN professores prof ON prof.id=m.professor_id
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

    let (doc, page1, layer1) = PdfDocument::new(&titulo, Mm(210.0), Mm(297.0), "Layer 1");
    // Use embedded LiberationSans (Unicode) instead of Helvetica (Latin-1 only).
    // This ensures math symbols like ≠, π, α, ≤, ≥, √ render correctly.
    let font      = doc.add_external_font(Cursor::new(FONT_REGULAR_BYTES)).map_err(|e| e.to_string())?;
    let font_bold = doc.add_external_font(Cursor::new(FONT_BOLD_BYTES)).map_err(|e| e.to_string())?;

    let mut layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 277.0_f32;
    let lm = 20.0_f32;
    let rm = 190.0_f32;

    // Helper macro: start a new page when needed, drawing footer on transition
    macro_rules! maybe_new_page {
        ($needed:expr) => {
            if y < ($needed as f32) {
                if !rodape.is_empty() {
                    layer.use_text(&rodape, 8.0, Mm(lm), Mm(14.0), &font);
                }
                let (np, nl) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                layer = doc.get_page(np).get_layer(nl);
                y = 277.0;
            }
        };
    }

    // ── LOGO ───────────────────────────────────────────────────────────────
    let logo_img: Option<(ic::DynamicImage, u32, u32)> = if !logo_path.is_empty() {
        ic::open(&logo_path).ok().map(|img| { let (w, h) = (img.width(), img.height()); (img, w, h) })
    } else { None };

    let has_logo = logo_img.is_some();
    let text_x   = if has_logo { 52.0_f32 } else { lm };

    if let Some((dyn_img, iw, ih)) = logo_img {
        let target_w_mm = 28.0_f32;
        let dpi         = 300.0_f32;
        let scale       = target_w_mm / ((iw as f32 / dpi) * 25.4);
        let logo_h_mm   = target_w_mm * ih as f32 / iw as f32;
        Image::from_dynamic_image(&dyn_img).add_to_layer(layer.clone(), ImageTransform {
            translate_x: Some(Mm(lm)),
            translate_y: Some(Mm(y - logo_h_mm)),
            dpi: Some(dpi),
            scale_x: Some(scale),
            scale_y: Some(scale),
            ..Default::default()
        });
    }

    // ── INSTITUTION BLOCK ──────────────────────────────────────────────────
    if !nome_escola.is_empty() {
        layer.use_text(&nome_escola, 14.0, Mm(text_x), Mm(y), &font_bold);
        y -= 8.0;
    }
    let data_cidade = match (!cidade.is_empty(), !data.is_empty()) {
        (true,  true)  => format!("{}, {}", cidade, format_date_pt(&data)),
        (true,  false) => cidade.clone(),
        (false, true)  => format_date_pt(&data),
        (false, false) => String::new(),
    };
    if !data_cidade.is_empty() {
        layer.use_text(&data_cidade, 10.0, Mm(text_x), Mm(y), &font);
        y -= 5.5;
    }
    if !diretor.is_empty() {
        layer.use_text(&format!("Diretor(a): {}", diretor), 10.0, Mm(text_x), Mm(y), &font);
        y -= 5.5;
    }
    if !professor.is_empty() {
        layer.use_text(&format!("Professor(a): {}", professor), 10.0, Mm(text_x), Mm(y), &font);
        y -= 5.5;
    }

    // Separator after institution block
    draw_hline(&layer, y - 1.5, lm, rm);
    y -= 7.0;

    // ── STUDENT FIELDS ─────────────────────────────────────────────────────
    layer.use_text(
        "Aluno(a): _____________________________________________",
        10.5, Mm(lm), Mm(y), &font,
    );
    y -= 6.5;
    layer.use_text(
        "Série/Ano: ___________  Turno: ___________  Valor: _________  Nota: _________",
        9.5, Mm(lm), Mm(y), &font,
    );
    y -= 5.0;

    draw_hline(&layer, y - 1.5, lm, rm);
    y -= 9.0;

    // ── EXAM TITLE ─────────────────────────────────────────────────────────
    layer.use_text(&titulo, 15.0, Mm(lm), Mm(y), &font_bold);
    y -= 8.5;

    if !descricao.is_empty() {
        for line in wrap_text(&descricao, 82) {
            maybe_new_page!(28.0);
            layer.use_text(&line, 10.0, Mm(lm), Mm(y), &font);
            y -= 5.5;
        }
    }

    draw_hline(&layer, y - 1.5, lm, rm);
    y -= 9.0;

    // ── QUESTIONS ──────────────────────────────────────────────────────────
    for (i, q) in questoes.iter().enumerate() {
        let opcoes_arr = q.opcoes.as_array();
        let opts_n = opcoes_arr.map(|o| o.len()).unwrap_or(0);
        let est_lines = match q.tipo.as_str() {
            "multipla_escolha" | "verdadeiro_falso" | "associacao" | "ordenar" => opts_n,
            _ => q.linhas_resposta as usize + 1,
        };
        maybe_new_page!(14.0 + est_lines as f32 * 6.5 + 10.0);

        // Question header
        layer.use_text(
            &format!("Questão {}   ({:.1} pt)", i + 1, q.valor),
            11.0, Mm(lm), Mm(y), &font_bold,
        );
        y -= 6.5;

        // Enunciado (word-wrapped)
        let enunciado_plain = html_to_plain(&q.enunciado);
        for line in wrap_text(&enunciado_plain, 80) {
            maybe_new_page!(25.0);
            layer.use_text(&line, 10.5, Mm(lm + 3.0), Mm(y), &font);
            y -= 6.0;
        }
        y -= 1.5; // extra gap before options

        match q.tipo.as_str() {
            "multipla_escolha" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            maybe_new_page!(18.0);
                            let letra = (b'a' + j as u8) as char;
                            layer.use_text(&format!("  {}) {}", letra, texto), 10.0, Mm(lm + 5.0), Mm(y), &font);
                            y -= 6.0;
                        }
                    }
                }
            }
            "verdadeiro_falso" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            maybe_new_page!(18.0);
                            let letra = (b'a' + j as u8) as char;
                            layer.use_text(
                                &format!("  {}) (  ) V    (  ) F    {}", letra, texto),
                                10.0, Mm(lm + 5.0), Mm(y), &font,
                            );
                            y -= 6.0;
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
                        for line in wrap_text(&format!("  Banco de palavras: {}", palavras.join("  |  ")), 82) {
                            maybe_new_page!(18.0);
                            layer.use_text(&line, 10.0, Mm(lm + 5.0), Mm(y), &font);
                            y -= 6.0;
                        }
                    }
                }
            }
            "associacao" => {
                if let Some(opcoes) = opcoes_arr {
                    for (j, o) in opcoes.iter().enumerate() {
                        let texto_a = o.get("texto").and_then(|t| t.as_str()).unwrap_or("");
                        let texto_b = o.get("par").and_then(|t| t.as_str()).unwrap_or("");
                        let letra   = (b'A' + j as u8) as char;
                        maybe_new_page!(18.0);
                        layer.use_text(&format!("  {}) {}", j + 1, texto_a), 10.0, Mm(lm + 5.0), Mm(y), &font);
                        layer.use_text(&format!("(  ) {}) {}", letra, texto_b), 10.0, Mm(118.0), Mm(y), &font);
                        y -= 6.0;
                    }
                }
            }
            "ordenar" => {
                if let Some(opcoes) = opcoes_arr {
                    for o in opcoes.iter() {
                        if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                            maybe_new_page!(18.0);
                            layer.use_text(&format!("  (   ) {}", texto), 10.0, Mm(lm + 5.0), Mm(y), &font);
                            y -= 6.0;
                        }
                    }
                }
            }
            _ => {
                for _ in 0..q.linhas_resposta {
                    maybe_new_page!(18.0);
                    layer.use_text(
                        "  _______________________________________________________________________",
                        10.0, Mm(lm + 3.0), Mm(y), &font,
                    );
                    y -= 6.5;
                }
            }
        }
        y -= 5.0; // gap between questions
    }

    // Footer on last page
    if !rodape.is_empty() {
        layer.use_text(&rodape, 8.0, Mm(lm), Mm(14.0), &font);
    }

    let file = File::create(&path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?;
    Ok(())
}
