/// Parses Tiptap HTML into a flat list of rich segments for export.
/// Math formulas ($...$) are preserved as their LaTeX source for text rendering.

use regex::Regex;
use scraper::{Html, Selector, ElementRef, Node};

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone)]
pub enum Block {
    /// A paragraph-level block containing inline spans
    Para {
        spans: Vec<TextSpan>,
        align: Align,
        heading: Option<u8>, // 1-6
    },
    /// A table row (cells are vecs of blocks)
    TableRow { cells: Vec<Vec<Block>>, is_header: bool },
    /// Horizontal rule
    Hr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align { Left, Center, Right }

/// Convert $...$ inline math LaTeX into a human-readable Unicode approximation.
/// For export purposes we show the LaTeX source wrapped in angle brackets so
/// users know it was a formula, e.g.: ⟨x² + 1⟩
pub fn latex_to_display(latex: &str) -> String {
    let s = latex.trim();
    // Apply common simple substitutions for a readable fallback
    let s = s.replace("\\frac{", "").replace("}{", "÷(").replace("}", ")");
    let s = s.replace("\\sqrt", "√");
    let s = s.replace("\\pi", "π");
    let s = s.replace("\\infty", "∞");
    let s = s.replace("\\Delta", "Δ");
    let s = s.replace("\\alpha", "α");
    let s = s.replace("\\beta", "β");
    let s = s.replace("\\gamma", "γ");
    let s = s.replace("\\theta", "θ");
    let s = s.replace("\\sigma", "σ");
    let s = s.replace("\\mu", "μ");
    let s = s.replace("\\lambda", "λ");
    let s = s.replace("\\omega", "ω");
    let s = s.replace("\\neq", "≠");
    let s = s.replace("\\leq", "≤");
    let s = s.replace("\\geq", "≥");
    let s = s.replace("\\in", "∈");
    let s = s.replace("\\notin", "∉");
    let s = s.replace("\\rightarrow", "→");
    let s = s.replace("\\leftarrow", "←");
    let s = s.replace("\\Leftrightarrow", "⟺");
    let s = s.replace("\\times", "×");
    let s = s.replace("\\div", "÷");
    let s = s.replace("\\cdot", "·");
    let s = s.replace("\\pm", "±");
    let s = s.replace("\\sum", "Σ");
    let s = s.replace("\\int", "∫");
    let s = s.replace("\\lim", "lim");
    let s = s.replace("\\log", "log");
    let s = s.replace("\\ln", "ln");
    let s = s.replace("\\vec{", "").replace("\\angle", "∠");
    let s = s.replace("\\circ", "°");
    let s = s.replace("\\bar{", "");
    let s = s.replace("\\begin{cases}", "[").replace("\\end{cases}", "]");
    let s = s.replace("\\\\", " ; ");
    let s = s.replace("_{", "_(").replace("^{", "^(");
    let s = s.replace("\\,", " ").replace("\\ ", " ");
    let s = Regex::new(r"\\[a-zA-Z]+").unwrap().replace_all(&s, "").to_string();
    // Clean up leftover braces
    let s = s.replace('{', "(").replace('}', ")");
    // Wrap in special brackets to mark it as formula
    format!("⟨{}⟩", s.trim())
}

/// Split raw text by $...$ math regions and produce spans
fn parse_inline_text(text: &str, bold: bool, italic: bool, underline: bool) -> Vec<TextSpan> {
    let re = Regex::new(r"\$([^$]+)\$").unwrap();
    let mut result = Vec::new();
    let mut last = 0;
    for cap in re.captures_iter(text) {
        let full = cap.get(0).unwrap();
        let latex = cap.get(1).unwrap().as_str();
        // Text before the formula
        let before = &text[last..full.start()];
        if !before.is_empty() {
            result.push(TextSpan { text: before.to_string(), bold, italic, underline });
        }
        result.push(TextSpan { text: latex_to_display(latex), bold: true, italic: false, underline: false });
        last = full.end();
    }
    let after = &text[last..];
    if !after.is_empty() {
        result.push(TextSpan { text: after.to_string(), bold, italic, underline });
    }
    if result.is_empty() { result.push(TextSpan { text: String::new(), bold, italic, underline }); }
    result
}

fn collect_spans(el: ElementRef, bold: bool, italic: bool, underline: bool) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => {
                spans.extend(parse_inline_text(t.as_ref(), bold, italic, underline));
            }
            Node::Element(e) => {
                let tag = e.name();
                let child_ref = ElementRef::wrap(child).unwrap();
                let (cb, ci, cu) = (
                    bold || matches!(tag, "strong" | "b"),
                    italic || matches!(tag, "em" | "i"),
                    underline || tag == "u",
                );
                spans.extend(collect_spans(child_ref, cb, ci, cu));
            }
            _ => {}
        }
    }
    spans
}

fn align_of(el: ElementRef) -> Align {
    let style = el.value().attr("style").unwrap_or("");
    if style.contains("center") { Align::Center }
    else if style.contains("right") { Align::Right }
    else { Align::Left }
}

fn parse_blocks(html: &str) -> Vec<Block> {
    let doc = Html::parse_fragment(html);
    let mut blocks = Vec::new();

    // We walk top-level children
    let body_sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, hr, table").unwrap();

    for el in doc.select(&body_sel) {
        let tag = el.value().name();
        match tag {
            "hr" => blocks.push(Block::Hr),
            "p" => {
                let spans = collect_spans(el, false, false, false);
                let align  = align_of(el);
                blocks.push(Block::Para { spans, align, heading: None });
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let lvl = tag.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                let spans = collect_spans(el, true, false, false);
                let align  = align_of(el);
                blocks.push(Block::Para { spans, align, heading: Some(lvl) });
            }
            "ul" | "ol" => {
                let li_sel = Selector::parse("li").unwrap();
                let ordered = tag == "ol";
                for (i, li) in el.select(&li_sel).enumerate() {
                    let prefix = if ordered { format!("{}. ", i + 1) } else { "• ".to_string() };
                    let mut spans = vec![TextSpan { text: prefix, bold: false, italic: false, underline: false }];
                    spans.extend(collect_spans(li, false, false, false));
                    blocks.push(Block::Para { spans, align: Align::Left, heading: None });
                }
            }
            "blockquote" => {
                let spans = collect_spans(el, false, true, false);
                blocks.push(Block::Para { spans, align: Align::Left, heading: None });
            }
            "table" => {
                let tr_sel = Selector::parse("tr").unwrap();
                let th_sel = Selector::parse("th").unwrap();
                let td_sel = Selector::parse("td, th").unwrap();
                for (ri, tr) in el.select(&tr_sel).enumerate() {
                    let is_header = tr.select(&th_sel).next().is_some() || ri == 0;
                    let cells: Vec<Vec<Block>> = tr.select(&td_sel)
                        .map(|cell| {
                            let txt = cell.text().collect::<String>();
                            let spans = parse_inline_text(&txt, false, false, false);
                            vec![Block::Para { spans, align: Align::Left, heading: None }]
                        })
                        .collect();
                    if !cells.is_empty() {
                        blocks.push(Block::TableRow { cells, is_header });
                    }
                }
            }
            _ => {}
        }
    }

    // If the HTML produced no blocks (e.g. bare text), treat it as one paragraph
    if blocks.is_empty() {
        let doc2 = Html::parse_fragment(html);
        let clean: String = doc2.root_element().text().collect();
        if !clean.is_empty() {
            blocks.push(Block::Para {
                spans: parse_inline_text(clean.trim(), false, false, false),
                align: Align::Left,
                heading: None,
            });
        }
    }
    blocks
}

/// Strip all HTML tags and $...$ math markers, returning plain text for
/// backwards-compatible simple text rendering.
pub fn html_to_plain(html: &str) -> String {
    let blocks = parse_blocks(html);
    let mut lines = Vec::new();
    for b in &blocks {
        match b {
            Block::Para { spans, .. } => {
                let line: String = spans.iter().map(|s| s.text.as_str()).collect();
                lines.push(line);
            }
            Block::Hr => lines.push("─────────────────────────────────────────────────────".to_string()),
            Block::TableRow { cells, .. } => {
                let row = cells.iter().map(|cell| {
                    cell.iter().map(|b| {
                        if let Block::Para { spans, .. } = b {
                            spans.iter().map(|s| s.text.as_str()).collect::<String>()
                        } else { String::new() }
                    }).collect::<Vec<String>>().join("")
                }).collect::<Vec<_>>().join("  |  ");
                lines.push(row);
            }
        }
    }
    lines.join("\n")
}

pub fn html_to_blocks(html: &str) -> Vec<Block> {
    parse_blocks(html)
}
