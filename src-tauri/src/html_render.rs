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

/// Convert $...$ inline math LaTeX into a human-readable Unicode approximation for text export.
pub fn latex_to_display(latex: &str) -> String {
    let s = latex.trim().to_string();

    // \frac{a}{b} → (a/b) — handle one level of nesting
    let re_frac = Regex::new(r"\\frac\{([^{}]*)\}\{([^{}]*)\}").unwrap();
    let mut s = re_frac.replace_all(&s, "($1/$2)").to_string();
    // Apply repeatedly for adjacent fracs
    while re_frac.is_match(&s) {
        s = re_frac.replace_all(&s, "($1/$2)").to_string();
    }

    // \sqrt{x} → √(x), \sqrt x → √
    let re_sqrt = Regex::new(r"\\sqrt\{([^{}]*)\}").unwrap();
    let s = re_sqrt.replace_all(&s, "√($1)").to_string();
    let s = s.replace("\\sqrt", "√");

    // Symbols — longer names first to avoid partial replacement
    let s = s.replace("\\Leftrightarrow", "⟺");
    let s = s.replace("\\rightarrow", "→");
    let s = s.replace("\\leftarrow", "←");
    let s = s.replace("\\notin", "∉");
    let s = s.replace("\\Delta", "Δ");
    let s = s.replace("\\Sigma", "Σ");
    let s = s.replace("\\alpha", "α");
    let s = s.replace("\\beta", "β");
    let s = s.replace("\\gamma", "γ");
    let s = s.replace("\\theta", "θ");
    let s = s.replace("\\sigma", "σ");
    let s = s.replace("\\lambda", "λ");
    let s = s.replace("\\omega", "ω");
    let s = s.replace("\\mu", "μ");
    let s = s.replace("\\pi", "π");
    let s = s.replace("\\infty", "∞");
    let s = s.replace("\\approx", "≈");
    let s = s.replace("\\neq", "≠");
    let s = s.replace("\\leq", "≤");
    let s = s.replace("\\geq", "≥");
    let s = s.replace("\\in", "∈");
    let s = s.replace("\\times", "×");
    let s = s.replace("\\div", "÷");
    let s = s.replace("\\cdot", "·");
    let s = s.replace("\\pm", "±");
    let s = s.replace("\\sum", "Σ");
    let s = s.replace("\\int", "∫");
    let s = s.replace("\\angle", "∠");
    let s = s.replace("\\circ", "°");
    let s = s.replace("\\lim", "lim");
    let s = s.replace("\\log", "log");
    let s = s.replace("\\ln", "ln");
    let s = s.replace("\\begin{cases}", "{").replace("\\end{cases}", "}");
    let s = s.replace("\\\\", "; ");
    let s = s.replace("\\,", " ").replace("\\ ", " ");

    // ^{expr} → ^expr  and _{expr} → _expr  (strip braces from exponents)
    let re_sup = Regex::new(r"\^\{([^{}]+)\}").unwrap();
    let s = re_sup.replace_all(&s, "^$1").to_string();
    let re_sub = Regex::new(r"_\{([^{}]+)\}").unwrap();
    let s = re_sub.replace_all(&s, "_$1").to_string();

    // Remove remaining LaTeX commands (\word)
    let s = Regex::new(r"\\[a-zA-Z]+").unwrap().replace_all(&s, "").to_string();

    // Clean up remaining stray braces
    let s = s.replace('{', "(").replace('}', ")");

    // Collapse multiple spaces
    let s = Regex::new(r" {2,}").unwrap().replace_all(s.trim(), " ").to_string();

    s
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
                // Tiptap Mathematics extension stores LaTeX in data-latex attribute.
                // <span data-type="inline-math" data-latex="..."> for inline math
                // <div  data-type="block-math"  data-latex="..."> for display math
                let data_type = e.attr("data-type").unwrap_or("");
                if data_type == "inline-math" || data_type == "block-math" {
                    if let Some(latex) = e.attr("data-latex") {
                        let rendered = latex_to_display(latex);
                        spans.push(TextSpan { text: rendered, bold: true, italic: false, underline: false });
                    }
                    continue;
                }
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

    // We walk top-level block elements.
    // div[data-type="block-math"] is Tiptap's block-level math node.
    let body_sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, hr, table, div[data-type=\"block-math\"]").unwrap();

    for el in doc.select(&body_sel) {
        let tag = el.value().name();
        match tag {
            "hr" => blocks.push(Block::Hr),
            "div" => {
                // Tiptap block math: <div data-type="block-math" data-latex="...">
                if let Some(latex) = el.value().attr("data-latex") {
                    let rendered = latex_to_display(latex);
                    blocks.push(Block::Para {
                        spans: vec![TextSpan { text: rendered, bold: true, italic: false, underline: false }],
                        align: Align::Center,
                        heading: None,
                    });
                }
            }
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

pub fn html_to_blocks(html: &str) -> Vec<Block> {
    parse_blocks(html)
}
