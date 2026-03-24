/// PDF export using the Typst engine — renders proper LaTeX math (fractions,
/// Greek letters, superscripts, etc.) rather than Unicode approximations.

use std::fs;
use std::collections::HashMap;
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
use crate::notas::get_nota_efetiva;
use rusqlite::params;

// ── Typst World implementation ──────────────────────────────────────────────

struct ExamWorld {
    source:  Source,
    library: LazyHash<Library>,
    book:    LazyHash<FontBook>,
    fonts:   Vec<Font>,
    files:   HashMap<String, Bytes>,  // Virtual files (images, etc.)
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
            files: HashMap::new(),
        }
    }
    
    fn new_with_files(markup: String, files: HashMap<String, Bytes>) -> Self {
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
            files,
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
        // Check if it's a virtual file (image).
        // On Windows the rooted path starts with '\' instead of '/',
        // so strip both separators to get the bare filename.
        let path = id.vpath().as_rooted_path().to_string_lossy().to_string();
        let clean_path = path.trim_start_matches(|c| c == '/' || c == '\\');
        if let Some(data) = self.files.get(clean_path) {
            return Ok(data.clone());
        }
        // Try to read from filesystem (absolute local paths stored verbatim)
        if let Ok(data) = fs::read(clean_path) {
            return Ok(Bytes::new(data));
        }
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

/// Decode base64 data URL to bytes
fn decode_data_url(url: &str) -> Option<(String, Vec<u8>)> {
    // data:image/png;base64,iVBORw0KGg...
    if !url.starts_with("data:") {
        return None;
    }
    let rest = url.strip_prefix("data:")?;
    let (mime_part, data) = rest.split_once(',')?;
    let mime = mime_part.split(';').next()?;
    let ext = match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "png",
    };
    
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let bytes = STANDARD.decode(data.trim()).ok()?;
    Some((ext.to_string(), bytes))
}

/// Convert Tiptap HTML enunciado to Typst markup, with proper math rendering.
/// Returns (typst_markup, images_map) where images_map contains virtual file paths → image bytes.
fn enunciado_to_typst_with_images(html: &str, image_counter: &mut u32) -> (String, HashMap<String, Bytes>) {
    use scraper::{Html as SHtml, Selector, ElementRef};

    let doc = SHtml::parse_fragment(html);
    let mut out = String::new();
    let mut images: HashMap<String, Bytes> = HashMap::new();

    // Walk all block-level elements in document order
    fn process_element(el: ElementRef, out: &mut String, images: &mut HashMap<String, Bytes>, counter: &mut u32) {
        let tag = el.value().name();

        match tag {
            // Images
            "img" => {
                if let Some(src) = el.value().attr("src") {
                    let (filename, bytes_opt) = if src.starts_with("data:") {
                        // Base64 encoded image
                        if let Some((ext, bytes)) = decode_data_url(src) {
                            *counter += 1;
                            let fname = format!("img_{}.{}", counter, ext);
                            (fname, Some(bytes))
                        } else {
                            return;
                        }
                    } else if src.starts_with("http://") || src.starts_with("https://") {
                        // URL image - skip for now (would need HTTP fetch)
                        return;
                    } else {
                        // Local file path
                        if let Ok(bytes) = fs::read(src) {
                            *counter += 1;
                            let ext = std::path::Path::new(src)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("png");
                            let fname = format!("img_{}.{}", counter, ext);
                            (fname, Some(bytes))
                        } else {
                            return;
                        }
                    };
                    
                    if let Some(bytes) = bytes_opt {
                        images.insert(filename.clone(), Bytes::new(bytes));
                        // Get width if specified
                        let width_attr = el.value().attr("width")
                            .or_else(|| {
                                el.value().attr("style").and_then(|s| {
                                    // Extract width from style="width: 300px"
                                    s.split(';')
                                        .find(|p| p.contains("width"))
                                        .and_then(|p| p.split(':').nth(1))
                                        .map(|w| w.trim().trim_end_matches("px"))
                                })
                            });
                        
                        if let Some(w) = width_attr {
                            if let Ok(wpx) = w.parse::<f64>() {
                                let width_pt = wpx * 0.75; // px to pt (approx)
                                out.push_str(&format!("#image(\"{}\", width: {}pt)\n\n", filename, width_pt));
                            } else {
                                out.push_str(&format!("#image(\"{}\", width: 60%)\n\n", filename));
                            }
                        } else {
                            out.push_str(&format!("#image(\"{}\", width: 60%)\n\n", filename));
                        }
                    }
                }
            }
            
            // Block math
            "div" => {
                if el.value().attr("data-type") == Some("block-math") {
                    if let Some(latex) = el.value().attr("data-latex") {
                        out.push_str(&format!("$ {} $\n\n", latex_to_typst(latex)));
                    }
                } else {
                    // Process children of generic divs
                    for child in el.children() {
                        if let Some(child_el) = ElementRef::wrap(child) {
                            process_element(child_el, out, images, counter);
                        }
                    }
                }
            }

            // Headings
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let n = tag.chars().nth(1).unwrap_or('1').to_digit(10).unwrap_or(2) as usize;
                let prefix = "=".repeat(n);
                let content = collect_inline_typst_with_images(el, images, counter);
                let align = get_alignment(&el);
                if align.is_empty() {
                    out.push_str(&format!("{} {}\n\n", prefix, content));
                } else {
                    out.push_str(&format!("#align({})[{} {}]\n\n", align, prefix, content));
                }
            }

            // Paragraphs with alignment support
            "p" => {
                let content = collect_inline_typst_with_images(el, images, counter);
                let align = get_alignment(&el);
                if align.is_empty() {
                    out.push_str(&format!("{}\n\n", content));
                } else {
                    out.push_str(&format!("#align({})[{}]\n\n", align, content));
                }
            }

            // Unordered lists
            "ul" => {
                for child in el.children() {
                    if let Some(li) = ElementRef::wrap(child) {
                        if li.value().name() == "li" {
                            let content = collect_inline_typst_with_images(li, images, counter);
                            out.push_str(&format!("- {}\n", content));
                        }
                    }
                }
                out.push('\n');
            }

            // Ordered lists
            "ol" => {
                let mut idx = 1;
                for child in el.children() {
                    if let Some(li) = ElementRef::wrap(child) {
                        if li.value().name() == "li" {
                            let content = collect_inline_typst_with_images(li, images, counter);
                            out.push_str(&format!("{}. {}\n", idx, content));
                            idx += 1;
                        }
                    }
                }
                out.push('\n');
            }

            // Blockquotes
            "blockquote" => {
                out.push_str("#block(inset: (left: 1em), stroke: (left: 2pt + gray))[\n");
                // Recursively process children
                for child in el.children() {
                    if let Some(child_el) = ElementRef::wrap(child) {
                        process_element(child_el, out, images, counter);
                    } else if let Some(text) = child.value().as_text() {
                        let t = text.trim();
                        if !t.is_empty() {
                            out.push_str(&escape_typst(t));
                            out.push('\n');
                        }
                    }
                }
                out.push_str("]\n\n");
            }

            // Tables
            "table" => {
                // Count columns from first row
                let tr_sel = Selector::parse("tr").unwrap();
                let th_td_sel = Selector::parse("th, td").unwrap();
                let rows: Vec<ElementRef> = el.select(&tr_sel).collect();
                if rows.is_empty() {
                    return;
                }
                let num_cols = rows[0].select(&th_td_sel).count();
                if num_cols == 0 {
                    return;
                }

                out.push_str(&format!("#table(columns: {},\n", num_cols));
                for row in &rows {
                    for cell in row.select(&th_td_sel) {
                        let content = collect_inline_typst_with_images(cell, images, counter);
                        let is_header = cell.value().name() == "th";
                        if is_header {
                            out.push_str(&format!("  table.header()[#strong[{}]],\n", content));
                        } else {
                            out.push_str(&format!("  [{}],\n", content));
                        }
                    }
                }
                out.push_str(")\n\n");
            }

            // Skip these — we process their children via parent
            "li" | "thead" | "tbody" | "tr" | "th" | "td" => {}

            // For other elements, try to get inline content
            _ => {
                let content = collect_inline_typst_with_images(el, images, counter);
                if !content.trim().is_empty() {
                    out.push_str(&format!("{}\n", content));
                }
            }
        }
    }

    // Get alignment from style attribute
    fn get_alignment(el: &ElementRef) -> String {
        if let Some(style) = el.value().attr("style") {
            if style.contains("text-align: center") || style.contains("text-align:center") {
                return "center".to_string();
            }
            if style.contains("text-align: right") || style.contains("text-align:right") {
                return "right".to_string();
            }
            if style.contains("text-align: justify") || style.contains("text-align:justify") {
                return "center".to_string(); // Typst doesn't have justify per-element easily
            }
        }
        String::new()
    }

    // Walk top-level elements
    let body_sel = Selector::parse("body, html").ok();
    let root = body_sel.as_ref()
        .and_then(|s| doc.select(s).next())
        .unwrap_or_else(|| doc.root_element());

    for child in root.children() {
        if let Some(el) = ElementRef::wrap(child) {
            process_element(el, &mut out, &mut images, image_counter);
        }
    }

    // If nothing parsed (bare text without block wrapper), fall back to plain text
    if out.trim().is_empty() {
        let plain: String = doc.root_element().text().collect();
        out = escape_typst(plain.trim());
    }

    (out, images)
}

/// Recursively collect inline Typst markup from an element, handling math nodes and images.
fn collect_inline_typst_with_images(el: scraper::ElementRef, images: &mut HashMap<String, Bytes>, counter: &mut u32) -> String {
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
                
                // Handle inline images
                if tag == "img" {
                    if let Some(src) = e.attr("src") {
                        let (filename, bytes_opt) = if src.starts_with("data:") {
                            if let Some((ext, bytes)) = decode_data_url(src) {
                                *counter += 1;
                                let fname = format!("img_{}.{}", counter, ext);
                                (fname, Some(bytes))
                            } else {
                                continue;
                            }
                        } else if !src.starts_with("http") {
                            if let Ok(bytes) = fs::read(src) {
                                *counter += 1;
                                let ext = std::path::Path::new(src)
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("png");
                                let fname = format!("img_{}.{}", counter, ext);
                                (fname, Some(bytes))
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };
                        
                        if let Some(bytes) = bytes_opt {
                            images.insert(filename.clone(), Bytes::new(bytes));
                            out.push_str(&format!("#image(\"{}\", height: 1em)", filename));
                        }
                    }
                    continue;
                }
                
                let child_ref = ElementRef::wrap(child).unwrap();
                let inner = collect_inline_typst_with_images(child_ref, images, counter);
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

// ── Build the full Typst document source ────────────────────────────────────

fn tema_to_primary_color(tema: &str) -> &'static str {
    match tema {
        "emerald" => "#66cc8a",
        "bumblebee" => "#e0a82e",
        "retro" => "#ef9995",
        "garden" => "#5c7f67",
        "forest" => "#1eb854",
        "aqua" => "#09ecf3",
        "business" => "#1c4f82",
        "night" | "dim" => "#38bdf8",
        "winter" => "#047AFF",
        "nord" => "#5e81ac",
        "sunset" | "autumn" => "#d97706",
        "coffee" => "#DB924B",
        "lemonade" => "#519903",
        "cyberpunk" => "#ff7598",
        _ => "#4b6bfb",
    }
}

/// Return a Typst background content expression that draws a page frame.
/// Uses absolute A4 dimensions (210×297 mm) with `m` = margem_folha offset from paper edge.
fn generate_frame_background(estilo: &str, m: f64) -> String {
    const FW: f64 = 210.0;
    const FH: f64 = 297.0;
    match estilo {
        "simple" => format!(
            "place(top + left, dx: {m:.2}mm, dy: {m:.2}mm, \
             rect(width: {w:.2}mm, height: {h:.2}mm, stroke: 0.5pt + black, fill: none))",
            m = m, w = FW - 2.0 * m, h = FH - 2.0 * m
        ),
        "double" => {
            let m2 = m + 3.0;
            format!(
                "{{ place(top + left, dx: {m:.2}mm, dy: {m:.2}mm, \
                 rect(width: {w1:.2}mm, height: {h1:.2}mm, stroke: 0.5pt + black, fill: none)); \
                 place(top + left, dx: {m2:.2}mm, dy: {m2:.2}mm, \
                 rect(width: {w2:.2}mm, height: {h2:.2}mm, stroke: 0.5pt + black, fill: none)) }}",
                m = m, w1 = FW - 2.0 * m,  h1 = FH - 2.0 * m,
                m2 = m2, w2 = FW - 2.0 * m2, h2 = FH - 2.0 * m2
            )
        }
        "ornate" => {
            let w  = FW - 2.0 * m;
            let h  = FH - 2.0 * m;
            let cs = 10.0_f64; // corner accent size mm
            let xr = m + w - cs;
            let yb = m + h - cs;
            format!(
                "{{ place(top + left, dx: {m:.2}mm, dy: {m:.2}mm, \
                 rect(width: {w:.2}mm, height: {h:.2}mm, stroke: 0.75pt + black, fill: none)); \
                 place(top + left, dx: {m:.2}mm, dy: {m:.2}mm, \
                 line(start: (0mm, {cs:.2}mm), end: ({cs:.2}mm, 0mm), stroke: 1.5pt + black)); \
                 place(top + left, dx: {xr:.2}mm, dy: {m:.2}mm, \
                 line(start: ({cs:.2}mm, 0mm), end: (0mm, {cs:.2}mm), stroke: 1.5pt + black)); \
                 place(top + left, dx: {m:.2}mm, dy: {yb:.2}mm, \
                 line(start: (0mm, 0mm), end: ({cs:.2}mm, {cs:.2}mm), stroke: 1.5pt + black)); \
                 place(top + left, dx: {xr:.2}mm, dy: {yb:.2}mm, \
                 line(start: (0mm, {cs:.2}mm), end: ({cs:.2}mm, 0mm), stroke: 1.5pt + black)) }}",
                m = m, w = w, h = h, cs = cs, xr = xr, yb = yb
            )
        }
        "classic" => {
            let m2 = m + 4.0;
            format!(
                "{{ place(top + left, dx: {m:.2}mm, dy: {m:.2}mm, \
                 rect(width: {w1:.2}mm, height: {h1:.2}mm, stroke: 1.5pt + black, fill: none)); \
                 place(top + left, dx: {m2:.2}mm, dy: {m2:.2}mm, \
                 rect(width: {w2:.2}mm, height: {h2:.2}mm, stroke: 0.3pt + black, fill: none)) }}",
                m = m, m2 = m2,
                w1 = FW - 2.0 * m,  h1 = FH - 2.0 * m,
                w2 = FW - 2.0 * m2, h2 = FH - 2.0 * m2
            )
        }
        "modern" => {
            let cs  = 15.0_f64; // corner L-shape length mm
            let t   =  0.7_f64; // ~2pt stroke thickness in mm
            let w   = FW - 2.0 * m;
            let h   = FH - 2.0 * m;
            let xr  = m + w - cs;
            let xrr = m + w - t;
            let yb  = m + h - t;
            let ybr = m + h - cs;
            format!(
                "{{ \
                 place(top+left, dx:{m:.2}mm,   dy:{m:.2}mm,  rect(width:{cs:.2}mm, height:{t:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{m:.2}mm,   dy:{m:.2}mm,  rect(width:{t:.2}mm, height:{cs:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{xr:.2}mm,  dy:{m:.2}mm,  rect(width:{cs:.2}mm, height:{t:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{xrr:.2}mm, dy:{m:.2}mm,  rect(width:{t:.2}mm, height:{cs:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{m:.2}mm,   dy:{yb:.2}mm, rect(width:{cs:.2}mm, height:{t:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{m:.2}mm,   dy:{ybr:.2}mm,rect(width:{t:.2}mm, height:{cs:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{xr:.2}mm,  dy:{yb:.2}mm, rect(width:{cs:.2}mm, height:{t:.2}mm, fill:black, stroke:none)); \
                 place(top+left, dx:{xrr:.2}mm, dy:{ybr:.2}mm,rect(width:{t:.2}mm, height:{cs:.2}mm, fill:black, stroke:none)) \
                 }}",
                m = m, cs = cs, t = t,
                xr = xr, xrr = xrr, yb = yb, ybr = ybr
            )
        }
        _ => String::new(), // "none" or unknown
    }
}

fn build_typst_source(
    titulo: &str,
    descricao: &str,
    nome_escola: &str,
    cidade: &str,
    professor: &str,
    logo_path: &str,
    cor_primaria: &str,
    questoes: &[Questao],
    moldura_estilo: &str,
    margem_folha: f64,
    margem_moldura: f64,
    margem_conteudo: f64,
    fonte: &str,
    tamanho_fonte: i64,
) -> (String, HashMap<String, Bytes>) {
    let mut src = String::new();
    let mut all_images: HashMap<String, Bytes> = HashMap::new();
    let mut image_counter = 0u32;

    let total_margin = margem_folha + margem_moldura + margem_conteudo;
    let bg_expr = if moldura_estilo != "none" {
        format!(", background: {}", generate_frame_background(moldura_estilo, margem_folha))
    } else {
        String::new()
    };

    src.push_str(&format!(
        "#set page(paper: \"a4\", margin: (left: {tm:.2}mm, right: {tm:.2}mm, top: {tm:.2}mm, bottom: {tm:.2}mm), numbering: \"1\"{bg})\n",
        tm = total_margin, bg = bg_expr
    ));
    src.push_str(&format!("#set text(font: \"{}\", size: {}pt)\n", fonte, tamanho_fonte));
    src.push_str("#set par(justify: false, leading: 0.65em)\n");
    src.push_str("#set math.equation(numbering: none)\n\n");

    // ── Decorative top band ──────────────────────────────────────────────────
    src.push_str(&format!(
        "#rect(width: 100%, height: 2.5mm, fill: rgb(\"{}\"), stroke: none)\n#v(1mm)\n",
        cor_primaria
    ));

    // ── Load logo ────────────────────────────────────────────────────────────
    let logo_ext = std::path::Path::new(logo_path)
        .extension().and_then(|e| e.to_str()).unwrap_or("png").to_string();
    let has_logo = if !logo_path.is_empty() {
        if let Ok(logo_bytes) = fs::read(logo_path) {
            all_images.insert(format!("logo_escola.{}", logo_ext), Bytes::new(logo_bytes));
            true
        } else { false }
    } else { false };

    let logo_col_content = if has_logo {
        format!("align(center + horizon)[#image(\"logo_escola.{}\", fit: \"contain\", height: 22mm)]", logo_ext)
    } else {
        "align(center + horizon)[]".to_string()
    };

    let cidade_data = if !cidade.is_empty() {
        format!("{}, ___/___", escape_typst(cidade))
    } else {
        "___/___".to_string()
    };
    let prof_fill = escape_typst(professor);

    // ── Two-column header grid ───────────────────────────────────────────────
    src.push_str(&format!(
        "#grid(\n  columns: (2fr, 8fr),\n  column-gutter: 4mm,\n  {},\n  block(width: 100%)[\n",
        logo_col_content
    ));
    if !nome_escola.is_empty() {
        src.push_str(&format!(
            "    #align(center)[#text(weight: \"bold\", size: 13pt)[{}]]\n",
            escape_typst(nome_escola)
        ));
    }
    if !cidade.is_empty() {
        src.push_str(&format!(
            "    #align(center)[#text(size: 9pt)[{}]]\n",
            escape_typst(cidade)
        ));
    }
    src.push_str("    #v(1.5mm)\n");
    src.push_str(&format!(
        "    #grid(\n      columns: (2.2cm, 1fr, 3.2cm, 1fr),\n      column-gutter: 1.5mm, row-gutter: 1.5mm,\n      [#text(size: 8pt, weight: \"bold\")[Data:]], [#text(size: 8pt)[{}]],\n      [#text(size: 8pt, weight: \"bold\")[Professor(a):]],[#box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[{}]],\n    )\n",
        cidade_data, prof_fill
    ));
    src.push_str("    #v(1mm)\n");
    src.push_str(
        "    #grid(\n      columns: (1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr),\n      column-gutter: 1mm, row-gutter: 1mm,\n      [#text(size: 8pt, weight: \"bold\")[Ano:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turma:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turno:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Valor:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Nota:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n    )\n"
    );
    src.push_str("  ]\n)\n");

    // ── Decorative bottom band ───────────────────────────────────────────────
    src.push_str(&format!(
        "#v(1mm)\n#rect(width: 100%, height: 1.5mm, fill: rgb(\"{}\"), stroke: none)\n#v(3mm)\n\n",
        cor_primaria
    ));

    // ── Title & description ──────────────────────────────────────────────────
    if !titulo.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 15pt, weight: \"bold\")[{}]]\n\n", escape_typst(titulo)));
    }
    if !descricao.is_empty() {
        let (desc_typst, desc_imgs) = enunciado_to_typst_with_images(descricao, &mut image_counter);
        all_images.extend(desc_imgs);
        src.push_str("#v(1mm)\n#align(center)[#emph[\n");
        src.push_str(&desc_typst);
        src.push_str("]]\n\n");
    }
    src.push_str("#line(length: 100%)\n\n");

    // ── Questions ────────────────────────────────────────────────────────────
    let mut questao_num = 0usize;
    for q in questoes.iter() {
        if q.tipo == "texto" {
            let (enunciado_typst, enunciado_images) = enunciado_to_typst_with_images(&q.enunciado, &mut image_counter);
            all_images.extend(enunciado_images);
            if !enunciado_typst.trim().is_empty() {
                src.push_str(&enunciado_typst);
                src.push_str("\n");
            }
            continue;
        }

        questao_num += 1;
        let valor_fmt = format!("{:.1}", q.valor);
        src.push_str(&format!(
            "#text(weight: \"bold\")[Questão {} ({} pt)]\n",
            questao_num, valor_fmt
        ));

        let (enunciado_typst, enunciado_images) = enunciado_to_typst_with_images(&q.enunciado, &mut image_counter);
        all_images.extend(enunciado_images);
        if !enunciado_typst.trim().is_empty() {
            src.push_str("#pad(left: 5mm)[\n");
            src.push_str(&enunciado_typst);
            src.push_str("]\n");
        }

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
                // dissertativa — blank spacing lines (no underlines, just vertical space)
                src.push_str("#pad(left: 5mm)[\n");
                for _ in 0..q.linhas_resposta {
                    src.push_str("#v(6mm)\n");
                }
                src.push_str("]\n");
            }
        }
        src.push_str("\n");
    }

    (src, all_images)
}

// ── Tauri command ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_prova_pdf(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, nome_escola, cidade, logo_path, tema,
         moldura_estilo, margem_folha, margem_moldura, margem_conteudo,
         fonte, tamanho_fonte, professor):
        (String, String, String, String, String, String, String, f64, f64, f64, String, i64, String) = conn.query_row(
        "SELECT p.titulo, p.descricao,
                COALESCE(c.nome_escola,''), COALESCE(c.cidade,''),
                COALESCE(c.logo_path,''), COALESCE(c.tema,'light'),
                COALESCE(c.moldura_estilo,'none'), COALESCE(c.margem_folha,15.0),
                COALESCE(c.margem_moldura,5.0), COALESCE(c.margem_conteudo,5.0),
                COALESCE(c.fonte,'New Computer Modern'), COALESCE(c.tamanho_fonte,11),
                COALESCE(prof.nome,'')
         FROM provas p
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=p.materia_id
         LEFT JOIN professores prof ON prof.id=m.professor_id
         WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                r.get(10)?, r.get(11)?, r.get(12)?)),
    ).map_err(|e| e.to_string())?;
    let cor_primaria = tema_to_primary_color(&tema).to_string();

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

    let (markup, images) = build_typst_source(
        &titulo, &descricao,
        &nome_escola, &cidade, &professor,
        &logo_path, &cor_primaria,
        &questoes,
        &moldura_estilo, margem_folha, margem_moldura, margem_conteudo,
        &fonte, tamanho_fonte,
    );

    let world = ExamWorld::new_with_files(markup.clone(), images);
    let _ = fs::write("/tmp/pedagoogle_last_export.typ", &markup);

    let result = typst::compile::<PagedDocument>(&world);

    let document = result.output.map_err(|errs| {
        let msg = errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ");
        log::error!("[typst error] {}", msg);
        msg
    })?;

    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|errs| errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; "))?;

    fs::write(&path, pdf_bytes).map_err(|e| e.to_string())
}

fn build_gabarito_source(titulo: &str, nome_escola: &str, questoes: &[Questao]) -> String {
    let mut src = String::new();
    src.push_str("#set page(paper: \"a4\", margin: 2cm, numbering: \"1\")\n");
    src.push_str("#set text(font: \"New Computer Modern\", size: 11pt)\n");
    src.push_str("#set par(leading: 0.65em)\n\n");
    if !nome_escola.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 12pt, weight: \"bold\")[{}]]\n", escape_typst(nome_escola)));
    }
    src.push_str("#align(center)[#text(size: 15pt, weight: \"bold\")[GABARITO]]\n");
    src.push_str(&format!("#align(center)[#text(size: 13pt)[{}]]\n", escape_typst(titulo)));
    src.push_str("\n#line(length: 100%)\n\n");
    for (i, q) in questoes.iter().enumerate() {
        let valor_fmt = format!("{:.1}", q.valor);
        src.push_str(&format!("#text(weight: \"bold\")[Questão {} ({} pt)]\n\n", i + 1, valor_fmt));
        let opcoes_arr = q.opcoes.as_array().map(|v| v.as_slice()).unwrap_or(&[]);
        match q.tipo.as_str() {
            "multipla_escolha" => {
                let idx = opcoes_arr.iter().position(|o| {
                    o.get("correta").and_then(|v| v.as_bool()).unwrap_or(false)
                });
                if let Some(idx) = idx {
                    let letra = (b'a' + idx as u8) as char;
                    src.push_str(&format!("#pad(left: 8mm)[Resposta: #strong[{}]]\n\n", letra));
                } else {
                    src.push_str("#pad(left: 8mm)[Resposta: —]\n\n");
                }
            }
            "verdadeiro_falso" => {
                src.push_str("#pad(left: 8mm)[\n");
                for (j, o) in opcoes_arr.iter().enumerate() {
                    let letra = (b'a' + j as u8) as char;
                    let vf = if o.get("correta").and_then(|v| v.as_bool()).unwrap_or(false) { "V" } else { "F" };
                    src.push_str(&format!("{}) #strong[{}]\n\n", letra, vf));
                }
                src.push_str("]\n\n");
            }
            "associacao" => {
                src.push_str("#pad(left: 8mm)[\n");
                for j in 0..opcoes_arr.len() {
                    let letra = (b'A' + j as u8) as char;
                    src.push_str(&format!("{} #sym.arrow.r #strong[{}]\n\n", j + 1, letra));
                }
                src.push_str("]\n\n");
            }
            "ordenar" => {
                src.push_str("#pad(left: 8mm)[\n");
                for (j, o) in opcoes_arr.iter().enumerate() {
                    if let Some(texto) = o.get("texto").and_then(|t| t.as_str()) {
                        src.push_str(&format!("{}) {}\n\n", j + 1, escape_typst(texto)));
                    }
                }
                src.push_str("]\n\n");
            }
            _ => {
                src.push_str("#pad(left: 8mm)[Gabarito: #line(length: 80%)]\n\n");
            }
        }
    }
    src
}

#[tauri::command]
pub fn export_gabarito_pdf(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;
    let (titulo, nome_escola): (String, String) = conn.query_row(
        "SELECT p.titulo, COALESCE(c.nome_escola,'') \
         FROM provas p LEFT JOIN configuracoes c ON c.id=1 WHERE p.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?)),
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
    let markup = build_gabarito_source(&titulo, &nome_escola, &questoes);
    let world = ExamWorld::new(markup);
    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|errs| {
        errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ")
    })?;
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|errs| errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; "))?;
    fs::write(&path, pdf_bytes).map_err(|e| e.to_string())
}

// ── Public function to render LaTeX as PNG ──────────────────────────────────

/// Render a LaTeX formula to PNG bytes using Typst.
/// Returns (png_bytes, width_px, height_px) or an error.
pub fn render_latex_to_png(latex: &str) -> Result<(Vec<u8>, u32, u32), String> {
    let typst_math = latex_to_typst(latex);
    
    // Create a minimal Typst document with just the formula
    // Use transparent background and auto page size
    let markup = format!(
        r#"#set page(width: auto, height: auto, margin: 2pt, fill: none)
#set text(font: "New Computer Modern", size: 14pt)
${}$"#,
        typst_math
    );
    
    let world = ExamWorld::new(markup);
    let result = typst::compile::<PagedDocument>(&world);
    
    let document = result.output.map_err(|errs| {
        let msg = errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ");
        log::error!("[typst latex error] {} -> {}", latex, msg);
        msg
    })?;
    
    // Render first page to pixmap at 2x scale for crisp rendering
    let page = &document.pages[0];
    let scale = 2.0;
    let pixmap = typst_render::render(page, scale);
    
    let width = pixmap.width();
    let height = pixmap.height();
    
    // Encode as PNG
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(pixmap.data()).map_err(|e| e.to_string())?;
    }
    
    Ok((png_data, width, height))
}

// ── Boletim PDF ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn export_boletim_pdf(aluno_id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let aluno_nome: String = conn.query_row(
        "SELECT nome FROM alunos WHERE id=?1",
        params![aluno_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;

    let (nome_escola, nota_minima, fonte, tamanho_fonte): (String, f64, String, i64) = conn.query_row(
        "SELECT COALESCE(nome_escola,''), COALESCE(nota_minima,5.0), COALESCE(fonte,'New Computer Modern'), COALESCE(tamanho_fonte,11) FROM configuracoes WHERE id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).unwrap_or_else(|_| (String::new(), 5.0, "New Computer Modern".into(), 11));

    let mut stmt = conn.prepare(
        "SELECT n.valor, COALESCE(p.titulo, n.descricao), COALESCE(p.valor_total, 1.0),
                COALESCE(m.id, -1), COALESCE(m.nome, 'Sem matéria')
         FROM notas n
         LEFT JOIN provas p ON p.id = n.prova_id
         LEFT JOIN materias m ON m.id = p.materia_id
         WHERE n.aluno_id = ?1
         ORDER BY m.nome, n.id",
    ).map_err(|e| e.to_string())?;

    struct NotaRow { valor: f64, titulo: String, peso: f64, materia_id: i64, materia_nome: String }
    let rows: Vec<NotaRow> = stmt.query_map(params![aluno_id], |r| {
        Ok(NotaRow { valor: r.get(0)?, titulo: r.get(1)?, peso: r.get(2)?, materia_id: r.get(3)?, materia_nome: r.get(4)? })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    struct GrupoMateria { nome: String, materia_id: i64, notas: Vec<(String, f64, f64)> }
    let mut grupos: Vec<GrupoMateria> = Vec::new();
    for row in &rows {
        if let Some(g) = grupos.iter_mut().find(|g| g.nome == row.materia_nome) {
            g.notas.push((row.titulo.clone(), row.valor, row.peso));
        } else {
            grupos.push(GrupoMateria { nome: row.materia_nome.clone(), materia_id: row.materia_id, notas: vec![(row.titulo.clone(), row.valor, row.peso)] });
        }
    }

    let mut src = String::new();
    src.push_str(&format!("#set page(paper: \"a4\", margin: 2cm, numbering: \"1\")\n"));
    src.push_str(&format!("#set text(font: \"{}\", size: {}pt)\n", fonte, tamanho_fonte));
    src.push_str("#set par(leading: 0.65em)\n\n");
    if !nome_escola.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 14pt, weight: \"bold\")[{}]]\n", escape_typst(&nome_escola)));
    }
    src.push_str("#align(center)[#text(size: 15pt, weight: \"bold\")[BOLETIM ESCOLAR]]\n");
    src.push_str(&format!("#align(center)[#text(size: 12pt)[Aluno(a): #strong[{}]]]\n", escape_typst(&aluno_nome)));
    src.push_str("\n#line(length: 100%)\n\n");

    src.push_str("#table(\n  columns: (2fr, 3fr, 1.2fr, 1.5fr),\n");
    src.push_str("  table.header()[#strong[Matéria]], table.header()[#strong[Provas e Notas]], table.header()[#strong[Média]], table.header()[#strong[Situação]],\n");

    for g in &grupos {
        let mut provas_str = String::new();
        for (titulo, valor, _peso) in &g.notas {
            if !provas_str.is_empty() { provas_str.push_str(" | "); }
            provas_str.push_str(&format!("{}: {:.1}", escape_typst(titulo), valor));
        }
        let media = if g.materia_id > 0 {
            get_nota_efetiva(&conn, aluno_id, g.materia_id)
        } else {
            let peso_total: f64 = g.notas.iter().map(|(_, _, p)| p).sum();
            let pond_total: f64 = g.notas.iter().map(|(_, v, p)| v * p).sum();
            if peso_total > 0.0 { pond_total / peso_total } else { 0.0 }
        };
        let situacao = if media >= nota_minima + 2.0 {
            "Aprovado"
        } else if media >= nota_minima {
            "Em Recup."
        } else {
            "Reprovado"
        };
        src.push_str(&format!(
            "  [{}], [{}], [#strong[{:.2}]], [{}],\n",
            escape_typst(&g.nome), provas_str, media, situacao
        ));
    }
    src.push_str(")\n");

    let world = ExamWorld::new(src);
    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|errs| {
        errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ")
    })?;
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|errs| errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; "))?;
    fs::write(&path, pdf_bytes).map_err(|e| e.to_string())
}

// ── Atividade PDF ─────────────────────────────────────────────────────────────

fn build_atividade_typst_source(
    titulo: &str,
    nome_materia: &str,
    descricao: &str,
    nome_escola: &str,
    cidade: &str,
    professor: &str,
    logo_path: &str,
    cor_primaria: &str,
    vale_nota: bool,
    valor_total: f64,
    questoes: &[crate::models::Questao],
    moldura_estilo: &str,
    margem_folha: f64,
    margem_moldura: f64,
    margem_conteudo: f64,
    fonte: &str,
    tamanho_fonte: i64,
) -> (String, HashMap<String, Bytes>) {
    let mut src = String::new();
    let mut all_images: HashMap<String, Bytes> = HashMap::new();
    let mut image_counter = 0u32;

    let total_margin = margem_folha + margem_moldura + margem_conteudo;
    let bg_expr = if moldura_estilo != "none" {
        format!(", background: {}", generate_frame_background(moldura_estilo, margem_folha))
    } else {
        String::new()
    };

    src.push_str(&format!(
        "#set page(paper: \"a4\", margin: (left: {tm:.2}mm, right: {tm:.2}mm, top: {tm:.2}mm, bottom: {tm:.2}mm), numbering: \"1\"{bg})\n",
        tm = total_margin, bg = bg_expr
    ));
    src.push_str(&format!("#set text(font: \"{}\", size: {}pt)\n", fonte, tamanho_fonte));
    src.push_str("#set par(justify: false, leading: 0.65em)\n");
    src.push_str("#set math.equation(numbering: none)\n\n");

    src.push_str(&format!(
        "#rect(width: 100%, height: 2.5mm, fill: rgb(\"{}\"), stroke: none)\n#v(1mm)\n",
        cor_primaria
    ));

    let logo_ext = std::path::Path::new(logo_path)
        .extension().and_then(|e| e.to_str()).unwrap_or("png").to_string();
    let has_logo = if !logo_path.is_empty() {
        if let Ok(logo_bytes) = fs::read(logo_path) {
            all_images.insert(format!("logo_escola.{}", logo_ext), Bytes::new(logo_bytes));
            true
        } else { false }
    } else { false };

    let logo_col_content = if has_logo {
        format!("align(center + horizon)[#image(\"logo_escola.{}\", fit: \"contain\", height: 22mm)]", logo_ext)
    } else {
        "align(center + horizon)[]".to_string()
    };

    let cidade_data = if !cidade.is_empty() {
        format!("{}, ___/___", escape_typst(cidade))
    } else {
        "___/___".to_string()
    };
    let prof_fill = escape_typst(professor);

    src.push_str(&format!(
        "#grid(\n  columns: (2fr, 8fr),\n  column-gutter: 4mm,\n  {},\n  block(width: 100%)[\n",
        logo_col_content
    ));
    if !nome_escola.is_empty() {
        src.push_str(&format!(
            "    #align(center)[#text(weight: \"bold\", size: 13pt)[{}]]\n",
            escape_typst(nome_escola)
        ));
    }
    if !cidade.is_empty() {
        src.push_str(&format!(
            "    #align(center)[#text(size: 9pt)[{}]]\n",
            escape_typst(cidade)
        ));
    }
    src.push_str("    #v(1.5mm)\n");
    src.push_str(&format!(
        "    #grid(\n      columns: (2.2cm, 1fr, 3.2cm, 1fr),\n      column-gutter: 1.5mm, row-gutter: 1.5mm,\n      [#text(size: 8pt, weight: \"bold\")[Data:]], [#text(size: 8pt)[{}]],\n      [#text(size: 8pt, weight: \"bold\")[Professor(a):]],[#box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[{}]],\n    )\n",
        cidade_data, prof_fill
    ));
    src.push_str("    #v(1mm)\n");
    if vale_nota {
        src.push_str(&format!(
            "    #grid(\n      columns: (1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr),\n      column-gutter: 1mm, row-gutter: 1mm,\n      [#text(size: 8pt, weight: \"bold\")[Ano:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turma:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turno:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Valor:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[#text(size: 8pt)[{:.1}]],\n      [#text(size: 8pt, weight: \"bold\")[Nota:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n    )\n",
            valor_total
        ));
    } else {
        src.push_str(
            "    #grid(\n      columns: (1.5cm, 1fr, 1.5cm, 1fr, 1.5cm, 1fr),\n      column-gutter: 1mm, row-gutter: 1mm,\n      [#text(size: 8pt, weight: \"bold\")[Ano:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turma:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n      [#text(size: 8pt, weight: \"bold\")[Turno:]], #box(width: 100%, height: 4mm, stroke: (bottom: 0.5pt + black))[],\n    )\n"
        );
    }
    src.push_str("  ]\n)\n");

    src.push_str(&format!(
        "#v(1mm)\n#rect(width: 100%, height: 1.5mm, fill: rgb(\"{}\"), stroke: none)\n#v(3mm)\n\n",
        cor_primaria
    ));

    if !nome_materia.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 10pt)[{}]]\n", escape_typst(nome_materia)));
    }
    if !titulo.is_empty() {
        src.push_str(&format!("#align(center)[#text(size: 15pt, weight: \"bold\")[{}]]\n\n", escape_typst(titulo)));
    }
    if !descricao.is_empty() {
        let (desc_typst, desc_imgs) = enunciado_to_typst_with_images(descricao, &mut image_counter);
        all_images.extend(desc_imgs);
        src.push_str("#v(1mm)\n#align(center)[#emph[\n");
        src.push_str(&desc_typst);
        src.push_str("]]\n\n");
    }
    src.push_str("#line(length: 100%)\n\n");

    let mut questao_num = 0usize;
    for q in questoes.iter() {
        if q.tipo == "texto" {
            let (enunciado_typst, enunciado_images) = enunciado_to_typst_with_images(&q.enunciado, &mut image_counter);
            all_images.extend(enunciado_images);
            if !enunciado_typst.trim().is_empty() {
                src.push_str(&enunciado_typst);
                src.push_str("\n");
            }
            continue;
        }
        questao_num += 1;
        let valor_fmt = format!("{:.1}", q.valor);
        src.push_str(&format!("#text(weight: \"bold\")[Questão {} ({} pt)]\n", questao_num, valor_fmt));
        let (enunciado_typst, enunciado_images) = enunciado_to_typst_with_images(&q.enunciado, &mut image_counter);
        all_images.extend(enunciado_images);
        if !enunciado_typst.trim().is_empty() {
            src.push_str("#pad(left: 5mm)[\n");
            src.push_str(&enunciado_typst);
            src.push_str("]\n");
        }
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
                src.push_str("#pad(left: 5mm)[\n");
                for _ in 0..q.linhas_resposta {
                    src.push_str("#v(6mm)\n");
                }
                src.push_str("]\n");
            }
        }
        src.push_str("\n");
    }

    (src, all_images)
}

#[tauri::command]
pub fn export_atividade_pdf(id: i64, path: String) -> Result<(), String> {
    let conn = get_conn().map_err(|e| e.to_string())?;

    let (titulo, descricao, nome_materia, nome_escola, cidade, logo_path, tema,
         moldura_estilo, margem_folha, margem_moldura, margem_conteudo,
         fonte, tamanho_fonte, professor, vale_nota, valor_total):
        (String, String, String, String, String, String, String, String, f64, f64, f64, String, i64, String, i64, f64) = conn.query_row(
        "SELECT a.titulo, a.descricao,
                COALESCE(m.nome,''), COALESCE(c.nome_escola,''), COALESCE(c.cidade,''),
                COALESCE(c.logo_path,''), COALESCE(c.tema,'light'),
                COALESCE(c.moldura_estilo,'none'), COALESCE(c.margem_folha,15.0),
                COALESCE(c.margem_moldura,5.0), COALESCE(c.margem_conteudo,5.0),
                COALESCE(c.fonte,'New Computer Modern'), COALESCE(c.tamanho_fonte,11),
                COALESCE(prof.nome,''),
                a.vale_nota, a.valor_total
         FROM atividades a
         LEFT JOIN configuracoes c ON c.id=1
         LEFT JOIN materias m ON m.id=a.materia_id
         LEFT JOIN professores prof ON prof.id=m.professor_id
         WHERE a.id=?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?, r.get(14)?, r.get(15)?)),
    ).map_err(|e| e.to_string())?;
    let cor_primaria = tema_to_primary_color(&tema).to_string();

    let mut stmt = conn.prepare(
        "SELECT id, atividade_id, enunciado, tipo, opcoes, ordem, valor, linhas_resposta \
         FROM questoes_atividade WHERE atividade_id=?1 ORDER BY ordem"
    ).map_err(|e| e.to_string())?;
    let questoes: Vec<crate::models::Questao> = stmt.query_map(params![id], |r| {
        let opcoes_str: String = r.get(4)?;
        Ok(crate::models::Questao {
            id: r.get(0)?, prova_id: r.get(1)?, enunciado: r.get(2)?,
            tipo: r.get(3)?,
            opcoes: serde_json::from_str(&opcoes_str).unwrap_or(serde_json::json!([])),
            ordem: r.get(5)?, valor: r.get(6)?, linhas_resposta: r.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let (markup, images) = build_atividade_typst_source(
        &titulo, &nome_materia, &descricao,
        &nome_escola, &cidade, &professor,
        &logo_path, &cor_primaria,
        vale_nota != 0, valor_total,
        &questoes,
        &moldura_estilo, margem_folha, margem_moldura, margem_conteudo,
        &fonte, tamanho_fonte,
    );

    let world = ExamWorld::new_with_files(markup.clone(), images);
    let _ = fs::write("/tmp/pedagoogle_last_export_atividade.typ", &markup);

    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|errs| {
        let msg = errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; ");
        log::error!("[typst error] {}", msg);
        msg
    })?;
    let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|errs| errs.iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("; "))?;
    fs::write(&path, pdf_bytes).map_err(|e| e.to_string())
}
