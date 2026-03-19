/// PDF export using the Typst engine — renders proper LaTeX math (fractions,
/// Greek letters, superscripts, etc.) rather than Unicode approximations.

use std::fs;
use regex::Regex;
use typst::LibraryExt;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_pdf::PdfOptions;
use crate::db::get_conn;
use crate::models::Questao;
use rusqlite::params;

// ── Typst World implementation ──────────────────────────────────────────────

struct ExamWorld {
    source:  Source,
    library: LazyHash<Library>,
    book:    LazyHash<FontBook>,
    fonts:   Vec<Font>,
}

impl ExamWorld {
    fn new(markup: String) -> Self {
        // Load all standard Typst bundled fonts (includes NewCMMath for math,
        // New Computer Modern for text, DejaVu Sans Mono for code, etc.)
        let fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        let book = FontBook::from_fonts(&fonts);
        let source = Source::detached(markup);
        Self {
            source,
            library: LazyHash::new(Library::default()),
            book:    LazyHash::new(book),
            fonts,
        }
    }
}

impl World for ExamWorld {
    fn library(&self) -> &LazyHash<Library> { &self.library }
    fn book(&self)    -> &LazyHash<FontBook> { &self.book }
    fn main(&self)    -> FileId { self.source.id() }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> { None }
}

// ── LaTeX → Typst math conversion ──────────────────────────────────────────

/// Convert LaTeX math syntax to Typst math syntax.
/// Typst math uses `$...$` delimiters (same as inline LaTeX) but different
/// command names. Most common school-math commands are covered here.
fn latex_to_typst(latex: &str) -> String {
    let s = latex.trim().to_string();

    // ── Handle LaTeX environments FIRST (before the catch-all stripping) ──

    // \begin{cases}...\end{cases} → cases(row1, row2, ...)
    let re_cases = Regex::new(r"(?s)\\begin\{cases\}(.*?)\\end\{cases\}").unwrap();
    let s = re_cases.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<&str> = inner.split("\\\\")
            .map(|r| r.trim())
            .filter(|r| !r.is_empty())
            .collect();
        format!("cases({})", rows.join(", "))
    }).to_string();

    // \begin{pmatrix}...\end{pmatrix} → mat(delim: "(", ...)
    let re_pmat = Regex::new(r"(?s)\\begin\{pmatrix\}(.*?)\\end\{pmatrix\}").unwrap();
    let s = re_pmat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ","))
            .filter(|r| !r.is_empty())
            .collect();
        format!("mat(delim: \"(\", {})", rows.join("; "))
    }).to_string();

    // \begin{bmatrix}...\end{bmatrix} → mat(delim: "[", ...)
    let re_bmat = Regex::new(r"(?s)\\begin\{bmatrix\}(.*?)\\end\{bmatrix\}").unwrap();
    let s = re_bmat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ","))
            .filter(|r| !r.is_empty())
            .collect();
        format!("mat(delim: \"[\", {})", rows.join("; "))
    }).to_string();

    // \begin{matrix}...\end{matrix} → mat(...)
    let re_mat = Regex::new(r"(?s)\\begin\{matrix\}(.*?)\\end\{matrix\}").unwrap();
    let s = re_mat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ","))
            .filter(|r| !r.is_empty())
            .collect();
        format!("mat({})", rows.join("; "))
    }).to_string();

    // \frac{a}{b} → a/b  (typst renders / as proper fraction in math mode)
    let re_frac = Regex::new(r"\\frac\{([^{}]*)\}\{([^{}]*)\}").unwrap();
    let mut s = re_frac.replace_all(&s, "($1)/($2)").to_string();
    while re_frac.is_match(&s) {
        s = re_frac.replace_all(&s, "($1)/($2)").to_string();
    }

    // \sqrt{x} → sqrt(x)
    let re_sqrt = Regex::new(r"\\sqrt\{([^{}]*)\}").unwrap();
    let s = re_sqrt.replace_all(&s, "sqrt($1)").to_string();
    let s = s.replace("\\sqrt", "sqrt");

    // x^{n} → x^(n) and x_{n} → x_(n)  — braces → parens for multi-char groups
    let re_sup = Regex::new(r"\^\{([^{}]+)\}").unwrap();
    let s = re_sup.replace_all(&s, "^($1)").to_string();
    let re_sub = Regex::new(r"_\{([^{}]+)\}").unwrap();
    let s = re_sub.replace_all(&s, "_($1)").to_string();

    // Longer command names FIRST to avoid partial replacement
    let s = s.replace("\\Leftrightarrow", "arrow.l.r");
    let s = s.replace("\\rightarrow", "->");
    let s = s.replace("\\leftarrow", "<-");
    let s = s.replace("\\notin", "in.not");
    let s = s.replace("\\neq",  "!=");
    let s = s.replace("\\leq",  "<=");
    let s = s.replace("\\geq",  ">=");
    let s = s.replace("\\approx", "approx");
    let s = s.replace("\\times", "times");
    let s = s.replace("\\div",  "div");
    let s = s.replace("\\cdot", "dot.op");
    let s = s.replace("\\pm",   "plus.minus");
    let s = s.replace("\\in",   "in");
    let s = s.replace("\\infty","oo");
    let s = s.replace("\\sum",  "sum");
    let s = s.replace("\\int",  "integral");
    let s = s.replace("\\lim",  "lim");
    let s = s.replace("\\log",  "log");
    let s = s.replace("\\ln",   "ln");
    let s = s.replace("\\angle","angle");
    let s = s.replace("\\circ", "circle.tiny");

    // Greek letters — typst uses names without backslash
    // \bar{x} → overline(x),  \vec{x} → arrow(x)
    let re_bar = Regex::new(r"\\bar\{([^{}]*)\}").unwrap();
    let s = re_bar.replace_all(&s, "overline($1)").to_string();
    let re_vec = Regex::new(r"\\vec\{([^{}]*)\}").unwrap();
    let s = re_vec.replace_all(&s, "arrow($1)").to_string();
    let s = s.replace("\\bar", "overline");
    let s = s.replace("\\vec", "arrow");

    let s = s.replace("\\Delta",  "Delta");
    let s = s.replace("\\Sigma",  "Sigma");
    let s = s.replace("\\Omega",  "Omega");
    let s = s.replace("\\Gamma",  "Gamma");
    let s = s.replace("\\Lambda", "Lambda");
    let s = s.replace("\\Theta",  "Theta");
    let s = s.replace("\\alpha",  "alpha");
    let s = s.replace("\\beta",   "beta");
    let s = s.replace("\\gamma",  "gamma");
    let s = s.replace("\\delta",  "delta");
    let s = s.replace("\\theta",  "theta");
    let s = s.replace("\\sigma",  "sigma");
    let s = s.replace("\\mu",     "mu");
    let s = s.replace("\\lambda", "lambda");
    let s = s.replace("\\omega",  "omega");
    let s = s.replace("\\pi",     "pi");

    let s = s.replace("\\\\", "; ");
    let s = s.replace("\\,",  " ").replace("\\ ", " ");

    // Remove any remaining unknown LaTeX commands
    let s = Regex::new(r"\\[a-zA-Z]+").unwrap().replace_all(&s, "").to_string();

    // Convert remaining braces to parens
    let s = s.replace('{', "(").replace('}', ")");

    // In Typst math, adjacent lowercase letters like "ax" are parsed as a single
    // unknown identifier. Insert spaces so "ax" → "a x" (implicit multiplication),
    // while preserving known Typst function/symbol names.
    let s = space_implicit_mult(&s);

    s.trim().to_string()
}

/// Insert spaces between adjacent lowercase letters that form unknown identifiers.
/// E.g. "ax" → "a x", "by" → "b y", but "sin", "sqrt", "cases" are preserved.
fn space_implicit_mult(s: &str) -> String {
    const KNOWN: &[&str] = &[
        "sin", "cos", "tan", "cot", "sec", "csc",
        "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
        "exp", "log", "ln", "lg", "det", "dim", "ker",
        "deg", "gcd", "lcm", "lim", "max", "min", "sup", "inf",
        "mod", "sqrt", "overline", "arrow", "cases", "mat",
        "integral", "sum", "prod", "approx", "dots", "oo", "div",
        "times", "pm", "mp", "in", "and", "or", "not",
        "alpha", "beta", "gamma", "delta", "epsilon", "varepsilon",
        "zeta", "eta", "theta", "vartheta", "iota", "kappa",
        "lambda", "mu", "nu", "xi", "pi", "varpi", "rho", "varrho",
        "sigma", "varsigma", "tau", "upsilon", "phi", "varphi", "chi",
        "psi", "omega",
    ];
    let re = Regex::new(r"[a-z]{2,}").unwrap();
    re.replace_all(s, |caps: &regex::Captures| {
        let word = caps.get(0).unwrap().as_str();
        if KNOWN.contains(&word) {
            word.to_string()
        } else {
            word.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
        }
    }).to_string()
}

// ── HTML/blocks → Typst markup ──────────────────────────────────────────────

/// Escape special Typst characters in plain text runs.
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('*', "\\*")
     .replace('_', "\\_")
     .replace('@', "\\@")
     .replace('#', "\\#")
     .replace('$', "\\$")
     .replace('<', "\\<")
     .replace('>', "\\>")
}



/// Convert Tiptap HTML enunciado to Typst markup, with proper math rendering.
/// Math nodes (`data-type="inline-math"` / `data-type="block-math"`) are
/// converted to Typst `$...$` inline math, which Typst renders properly.
fn enunciado_to_typst(html: &str) -> String {
    use scraper::{Html as SHtml, Selector};
    // use regex::Regex; // Removed unused import

    let doc = SHtml::parse_fragment(html);
    let mut out = String::new();

    // Walk top-level block elements
    let sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, div[data-type=\"block-math\"]").unwrap();
    for el in doc.select(&sel) {
        let tag = el.value().name();
        if tag == "div" {
            if let Some(latex) = el.value().attr("data-latex") {
                out.push_str(&format!("$ {} $\n", latex_to_typst(latex)));
            }
            continue;
        }
        // Paragraph / heading — walk inline children
        let content = collect_inline_typst(el);
        let lvl_prefix = if tag.starts_with('h') {
            let n = tag.chars().nth(1).unwrap_or('1').to_digit(10).unwrap_or(2) as usize;
            "=".repeat(n) + " "
        } else {
            String::new()
        };
        out.push_str(&lvl_prefix);
        out.push_str(&content);
        out.push('\n');
    }

    // If nothing parsed (bare text without block wrapper), fall back to plain text
    if out.trim().is_empty() {
        let plain: String = doc.root_element().text().collect();
        out = escape_typst(plain.trim());
    }

    out
}

/// Recursively collect inline Typst markup from an element, handling math nodes.
fn collect_inline_typst(el: scraper::ElementRef) -> String {
    use scraper::{Node, ElementRef};
    let mut out = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => {
                // Also handle legacy $...$ plain text format for old DB data
                let re = Regex::new(r"\$([^$]+)\$").unwrap();
                let text = t.as_ref();
                let mut last = 0;
                for cap in re.captures_iter(text) {
                    let full = cap.get(0).unwrap();
                    let latex = cap.get(1).unwrap().as_str();
                    let before = &text[last..full.start()];
                    if !before.is_empty() { out.push_str(&escape_typst(before)); }
                    out.push_str(&format!("${}$", latex_to_typst(latex)));
                    last = full.end();
                }
                let after = &text[last..];
                if !after.is_empty() { out.push_str(&escape_typst(after)); }
            }
            Node::Element(e) => {
                let data_type = e.attr("data-type").unwrap_or("");
                // Tiptap inline math: <span data-type="inline-math" data-latex="...">
                if data_type == "inline-math" {
                    if let Some(latex) = e.attr("data-latex") {
                        out.push_str(&format!("${}$", latex_to_typst(latex)));
                    }
                    continue;
                }
                let tag = e.name();
                let child_ref = ElementRef::wrap(child).unwrap();
                let inner = collect_inline_typst(child_ref);
                match tag {
                    "strong" | "b" => out.push_str(&format!("#strong[{}]", inner)),
                    "em"     | "i" => out.push_str(&format!("#emph[{}]", inner)),
                    "u"            => out.push_str(&format!("#underline[{}]", inner)),
                    _              => out.push_str(&inner),
                }
            }
            _ => {}
        }
    }
    out
}

fn format_date_pt(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let dia: u32 = parts[2].parse().unwrap_or(0);
    let mes = match parts[1] {
        "01" => "janeiro",  "02" => "fevereiro", "03" => "março",
        "04" => "abril",    "05" => "maio",       "06" => "junho",
        "07" => "julho",    "08" => "agosto",     "09" => "setembro",
        "10" => "outubro",  "11" => "novembro",   "12" => "dezembro",
        _ => parts[1],
    };
    format!("{} de {} de {}", dia, mes, parts[0])
}

// ── Build the full Typst document source ────────────────────────────────────

fn build_typst_source(
    titulo: &str, descricao: &str, rodape: &str,
    nome_escola: &str, cidade: &str, diretor: &str, professor: &str, data: &str,
    questoes: &[Questao],
) -> String {
    let mut src = String::new();

    // Page setup
    src.push_str("#set page(paper: \"a4\", margin: (left: 20mm, right: 20mm, top: 15mm, bottom: 20mm))\n");
    src.push_str("#set text(font: \"New Computer Modern\", size: 11pt)\n");
    src.push_str("#set par(justify: false, leading: 0.65em)\n");
    src.push_str("#set math.equation(numbering: none)\n\n");

    // Header block
    if !nome_escola.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 14pt, weight: \"bold\")[{}]]\n", escape_typst(nome_escola)));
    }
    let data_cidade = match (!cidade.is_empty(), !data.is_empty()) {
        (true, true)   => format!("{}, {}", cidade, format_date_pt(data)),
        (true, false)  => cidade.to_string(),
        (false, true)  => format_date_pt(data),
        (false, false) => String::new(),
    };
    if !data_cidade.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 10pt)[{}]]\n", escape_typst(&data_cidade)));
    }
    if !diretor.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 10pt)[Diretor(a): {}]]\n", escape_typst(diretor)));
    }
    if !professor.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 10pt)[Professor(a): {}]]\n", escape_typst(professor)));
    }
    src.push_str("\n#line(length: 100%)\n\n");

    // Student fields
    src.push_str("Aluno(a): #underline[#h(7cm)] Nota: #underline[#h(2cm)]\n\n");
    src.push_str("Série/Ano: #underline[#h(2.5cm)] Turno: #underline[#h(2.5cm)] Valor: #underline[#h(2cm)]\n\n");
    src.push_str("#line(length: 100%)\n\n");

    // Exam title
    if !titulo.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 15pt, weight: \"bold\")[{}]]\n\n", escape_typst(titulo)));
    }
    if !descricao.is_empty() {
        src.push_str(&format!("#align(center)[#emph[{}]]\n\n", escape_typst(descricao)));
    }
    src.push_str("#line(length: 100%)\n\n");

    // Questions
    for (i, q) in questoes.iter().enumerate() {
        let valor_fmt = format!("{:.1}", q.valor);
        src.push_str(&format!(
            "#text(weight: \"bold\")[Questão {} ({} pt)]\n",
            i + 1, valor_fmt
        ));

        // Enunciado — convert HTML with math to Typst markup
        let enunciado_typst = enunciado_to_typst(&q.enunciado);
        if !enunciado_typst.trim().is_empty() {
            src.push_str("#pad(left: 5mm)[\n");
            src.push_str(&enunciado_typst);
            src.push_str("]\n");
        }

        // Options / answer area
        let opcoes_arr = q.opcoes.as_array().map(|v| v.as_slice()).unwrap_or(&[]);
        match q.tipo.as_str() {
            "multipla_escolha" => {
                src.push_str("#pad(left: 8mm)[\n");
                for (j, o) in opcoes_arr.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        let letra = (b'a' + j as u8) as char;
                        src.push_str(&format!("{}) {}\n\n", letra, escape_typst(texto)));
                    }
                }
                src.push_str("]\n");
            }
            "verdadeiro_falso" => {
                src.push_str("#pad(left: 8mm)[\n");
                for (j, o) in opcoes_arr.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        let letra = (b'a' + j as u8) as char;
                        src.push_str(&format!("{}) (  ) V #h(3mm) (  ) F #h(3mm) {}\n\n", letra, escape_typst(texto)));
                    }
                }
                src.push_str("]\n");
            }
            "completar_lacunas" => {
                let palavras: Vec<&str> = opcoes_arr.iter()
                    .filter_map(|o| o.get("texto").and_then(|t| t.as_str()))
                    .collect();
                if !palavras.is_empty() {
                    let joined = palavras.iter().map(|p| escape_typst(p)).collect::<Vec<_>>().join(" | ");
                    src.push_str(&format!("#pad(left: 8mm)[Banco de palavras: {}]\n", joined));
                }
            }
            "associacao" => {
                src.push_str("#pad(left: 8mm)[\n");
                for (j, o) in opcoes_arr.iter().enumerate() {
                    let a = o.get("texto").and_then(|t| t.as_str()).unwrap_or("");
                    let b_text = o.get("par").and_then(|t| t.as_str()).unwrap_or("");
                    let letra = (b'A' + j as u8) as char;
                    src.push_str(&format!(
                        "{}) #h(1mm) {} #h(5mm) (  ) {}) {}\n\n",
                        j + 1, escape_typst(a), letra, escape_typst(b_text)
                    ));
                }
                src.push_str("]\n");
            }
            "ordenar" => {
                src.push_str("#pad(left: 8mm)[\n");
                for o in opcoes_arr.iter() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        src.push_str(&format!("(   ) {}\n\n", escape_typst(texto)));
                    }
                }
                src.push_str("]\n");
            }
            _ => {
                // dissertativa — answer lines
                src.push_str("#pad(left: 5mm)[\n");
                for _ in 0..q.linhas_resposta {
                    src.push_str("#line(length: 100%)\n\n");
                }
                src.push_str("]\n");
            }
        }
        src.push_str("\n");
    }

    // Footer
    if !rodape.is_empty() {
        src.push_str("#line(length: 100%)\n");
        src.push_str(&format!("#align(center)[#text(size: 9pt)[{}]]\n", escape_typst(rodape)));
    }

    src
}

// ── Tauri command ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_prova_pdf(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, rodape, nome_escola, cidade, diretor, professor, data, _logo_path):
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

    let markup = build_typst_source(
        &titulo, &descricao, &rodape,
        &nome_escola, &cidade, &diretor, &professor, &data,
        &questoes,
    );

    let world = ExamWorld::new(markup.clone());
    // Write typst source to /tmp for debugging — remove after confirming PDF is correct
    let _ = fs::write("/tmp/pedagoogle_last_export.typ", &markup);

    let result = typst::compile::<PagedDocument>(&world);

    let document = result.output.map_err(|errs| {
        let msg = errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ");
        eprintln!("[typst error] {}", msg);
        msg
    })?;

    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|errs| errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; "))?;

    fs::write(&path, pdf_bytes).map_err(|e| e.to_string())
}
