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
    pub is_math: bool,      // If true, text contains LaTeX to be rendered
    pub latex: Option<String>, // Original LaTeX source
}

#[derive(Debug, Clone)]
pub enum Block {
    /// A paragraph-level block containing inline spans
    Para {
        spans: Vec<TextSpan>,
        align: Align,
        heading: Option<u8>, // 1-6
    },
    /// A complete table with rows
    Table { rows: Vec<TableRowData> },
    /// Horizontal rule
    Hr,
    /// Image with base64 data or path
    Image {
        data: Vec<u8>,
        width: Option<u32>,  // pixels
        #[allow(dead_code)]
        height: Option<u32>, // pixels - reserved for future use
    },
}

#[derive(Debug, Clone)]
pub struct TableRowData {
    pub cells: Vec<String>,
    pub is_header: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align { Left, Center, Right }

/// Convert $...$ inline math LaTeX into a human-readable Unicode approximation for text export.
pub fn latex_to_display(latex: &str) -> String {
    let s = latex.trim().to_string();

    // ── Handle environments FIRST ──────────────────────────────────────────
    
    // \begin{cases}...\end{cases} → { condition1; condition2; ... }
    let re_cases = Regex::new(r"(?s)\\begin\{cases\}(.*?)\\end\{cases\}").unwrap();
    let s = re_cases.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", " se "))
            .filter(|r| !r.is_empty())
            .collect();
        format!("{{ {} }}", rows.join("; "))
    }).to_string();

    // \begin{pmatrix}...\end{pmatrix} → [r1; r2; ...]
    let re_pmat = Regex::new(r"(?s)\\begin\{pmatrix\}(.*?)\\end\{pmatrix\}").unwrap();
    let s = re_pmat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ", "))
            .filter(|r| !r.is_empty())
            .collect();
        format!("({})", rows.join("; "))
    }).to_string();

    // \begin{bmatrix}...\end{bmatrix} → [r1; r2; ...]
    let re_bmat = Regex::new(r"(?s)\\begin\{bmatrix\}(.*?)\\end\{bmatrix\}").unwrap();
    let s = re_bmat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ", "))
            .filter(|r| !r.is_empty())
            .collect();
        format!("[{}]", rows.join("; "))
    }).to_string();

    // \begin{matrix}...\end{matrix} → [r1; r2; ...]
    let re_mat = Regex::new(r"(?s)\\begin\{matrix\}(.*?)\\end\{matrix\}").unwrap();
    let s = re_mat.replace_all(&s, |caps: &regex::Captures| {
        let inner = caps.get(1).unwrap().as_str();
        let rows: Vec<String> = inner.split("\\\\")
            .map(|r| r.trim().replace("&", ", "))
            .filter(|r| !r.is_empty())
            .collect();
        format!("[{}]", rows.join("; "))
    }).to_string();

    // ── Fractions ──────────────────────────────────────────────────────────
    
    // \frac{a}{b} → a/b — handle nested fractions
    let re_frac = Regex::new(r"\\frac\{([^{}]*)\}\{([^{}]*)\}").unwrap();
    let mut s = re_frac.replace_all(&s, "($1)/($2)").to_string();
    // Apply repeatedly for nested fracs
    for _ in 0..5 {
        if !re_frac.is_match(&s) { break; }
        s = re_frac.replace_all(&s, "($1)/($2)").to_string();
    }
    // Simplify simple fractions like (a)/(b) where a,b are single chars
    let re_simple_frac = Regex::new(r"\(([a-zA-Z0-9])\)/\(([a-zA-Z0-9])\)").unwrap();
    let s = re_simple_frac.replace_all(&s, "$1/$2").to_string();

    // ── Square root ────────────────────────────────────────────────────────
    
    // \sqrt{x} → √(x), \sqrt[n]{x} → ⁿ√(x)
    let re_sqrt_n = Regex::new(r"\\sqrt\[([^\]]+)\]\{([^{}]*)\}").unwrap();
    let s = re_sqrt_n.replace_all(&s, |caps: &regex::Captures| {
        let n = caps.get(1).unwrap().as_str();
        let x = caps.get(2).unwrap().as_str();
        format!("{}√({})", to_superscript(n), x)
    }).to_string();
    let re_sqrt = Regex::new(r"\\sqrt\{([^{}]*)\}").unwrap();
    let s = re_sqrt.replace_all(&s, "√($1)").to_string();
    let s = s.replace("\\sqrt", "√");

    // ── Superscripts and subscripts ────────────────────────────────────────
    
    // ^{expr} → convert to Unicode superscript where possible
    let re_sup = Regex::new(r"\^\{([^{}]+)\}").unwrap();
    let s = re_sup.replace_all(&s, |caps: &regex::Captures| {
        to_superscript(caps.get(1).unwrap().as_str())
    }).to_string();
    // ^single_char
    let re_sup_single = Regex::new(r"\^([a-zA-Z0-9])").unwrap();
    let s = re_sup_single.replace_all(&s, |caps: &regex::Captures| {
        to_superscript(caps.get(1).unwrap().as_str())
    }).to_string();
    
    // _{expr} → convert to Unicode subscript where possible
    let re_sub = Regex::new(r"_\{([^{}]+)\}").unwrap();
    let s = re_sub.replace_all(&s, |caps: &regex::Captures| {
        to_subscript(caps.get(1).unwrap().as_str())
    }).to_string();
    // _single_char
    let re_sub_single = Regex::new(r"_([a-zA-Z0-9])").unwrap();
    let s = re_sub_single.replace_all(&s, |caps: &regex::Captures| {
        to_subscript(caps.get(1).unwrap().as_str())
    }).to_string();

    // ── Operators and symbols (longer names first) ─────────────────────────
    
    let s = s.replace("\\Leftrightarrow", "⟺");
    let s = s.replace("\\leftrightarrow", "↔");
    let s = s.replace("\\Rightarrow", "⟹");
    let s = s.replace("\\rightarrow", "→");
    let s = s.replace("\\Leftarrow", "⟸");
    let s = s.replace("\\leftarrow", "←");
    let s = s.replace("\\uparrow", "↑");
    let s = s.replace("\\downarrow", "↓");
    let s = s.replace("\\therefore", "∴");
    let s = s.replace("\\because", "∵");
    let s = s.replace("\\forall", "∀");
    let s = s.replace("\\exists", "∃");
    let s = s.replace("\\nexists", "∄");
    let s = s.replace("\\emptyset", "∅");
    let s = s.replace("\\varnothing", "∅");
    let s = s.replace("\\infty", "∞");
    let s = s.replace("\\partial", "∂");
    let s = s.replace("\\nabla", "∇");
    
    // Comparison operators
    let s = s.replace("\\approx", "≈");
    let s = s.replace("\\equiv", "≡");
    let s = s.replace("\\cong", "≅");
    let s = s.replace("\\sim", "∼");
    let s = s.replace("\\simeq", "≃");
    let s = s.replace("\\neq", "≠");
    let s = s.replace("\\ne", "≠");
    let s = s.replace("\\leq", "≤");
    let s = s.replace("\\le", "≤");
    let s = s.replace("\\geq", "≥");
    let s = s.replace("\\ge", "≥");
    let s = s.replace("\\ll", "≪");
    let s = s.replace("\\gg", "≫");
    let s = s.replace("\\prec", "≺");
    let s = s.replace("\\succ", "≻");
    
    // Set operations
    let s = s.replace("\\subset", "⊂");
    let s = s.replace("\\supset", "⊃");
    let s = s.replace("\\subseteq", "⊆");
    let s = s.replace("\\supseteq", "⊇");
    let s = s.replace("\\notin", "∉");
    let s = s.replace("\\in", "∈");
    let s = s.replace("\\cup", "∪");
    let s = s.replace("\\cap", "∩");
    let s = s.replace("\\setminus", "∖");
    
    // Arithmetic operators
    let s = s.replace("\\times", "×");
    let s = s.replace("\\div", "÷");
    let s = s.replace("\\cdot", "·");
    let s = s.replace("\\ast", "∗");
    let s = s.replace("\\star", "⋆");
    let s = s.replace("\\pm", "±");
    let s = s.replace("\\mp", "∓");
    let s = s.replace("\\oplus", "⊕");
    let s = s.replace("\\otimes", "⊗");
    
    // Big operators
    let s = s.replace("\\sum", "Σ");
    let s = s.replace("\\prod", "Π");
    let s = s.replace("\\coprod", "∐");
    let s = s.replace("\\int", "∫");
    let s = s.replace("\\iint", "∬");
    let s = s.replace("\\iiint", "∭");
    let s = s.replace("\\oint", "∮");
    
    // Functions (keep as text)
    let s = s.replace("\\arcsin", "arcsin");
    let s = s.replace("\\arccos", "arccos");
    let s = s.replace("\\arctan", "arctan");
    let s = s.replace("\\sinh", "sinh");
    let s = s.replace("\\cosh", "cosh");
    let s = s.replace("\\tanh", "tanh");
    let s = s.replace("\\sin", "sin");
    let s = s.replace("\\cos", "cos");
    let s = s.replace("\\tan", "tan");
    let s = s.replace("\\cot", "cot");
    let s = s.replace("\\sec", "sec");
    let s = s.replace("\\csc", "csc");
    let s = s.replace("\\lim", "lim");
    let s = s.replace("\\log", "log");
    let s = s.replace("\\ln", "ln");
    let s = s.replace("\\exp", "exp");
    let s = s.replace("\\max", "max");
    let s = s.replace("\\min", "min");
    let s = s.replace("\\sup", "sup");
    let s = s.replace("\\inf", "inf");
    let s = s.replace("\\det", "det");
    let s = s.replace("\\gcd", "gcd");
    let s = s.replace("\\lcm", "lcm");
    let s = s.replace("\\mod", "mod");
    
    // Geometry
    let s = s.replace("\\angle", "∠");
    let s = s.replace("\\measuredangle", "∡");
    let s = s.replace("\\triangle", "△");
    let s = s.replace("\\square", "□");
    let s = s.replace("\\circ", "°");
    let s = s.replace("\\perp", "⊥");
    let s = s.replace("\\parallel", "∥");
    
    // Greek letters - uppercase
    let s = s.replace("\\Alpha", "Α");
    let s = s.replace("\\Beta", "Β");
    let s = s.replace("\\Gamma", "Γ");
    let s = s.replace("\\Delta", "Δ");
    let s = s.replace("\\Epsilon", "Ε");
    let s = s.replace("\\Zeta", "Ζ");
    let s = s.replace("\\Eta", "Η");
    let s = s.replace("\\Theta", "Θ");
    let s = s.replace("\\Iota", "Ι");
    let s = s.replace("\\Kappa", "Κ");
    let s = s.replace("\\Lambda", "Λ");
    let s = s.replace("\\Mu", "Μ");
    let s = s.replace("\\Nu", "Ν");
    let s = s.replace("\\Xi", "Ξ");
    let s = s.replace("\\Omicron", "Ο");
    let s = s.replace("\\Pi", "Π");
    let s = s.replace("\\Rho", "Ρ");
    let s = s.replace("\\Sigma", "Σ");
    let s = s.replace("\\Tau", "Τ");
    let s = s.replace("\\Upsilon", "Υ");
    let s = s.replace("\\Phi", "Φ");
    let s = s.replace("\\Chi", "Χ");
    let s = s.replace("\\Psi", "Ψ");
    let s = s.replace("\\Omega", "Ω");
    
    // Greek letters - lowercase
    let s = s.replace("\\alpha", "α");
    let s = s.replace("\\beta", "β");
    let s = s.replace("\\gamma", "γ");
    let s = s.replace("\\delta", "δ");
    let s = s.replace("\\epsilon", "ε");
    let s = s.replace("\\varepsilon", "ε");
    let s = s.replace("\\zeta", "ζ");
    let s = s.replace("\\eta", "η");
    let s = s.replace("\\theta", "θ");
    let s = s.replace("\\vartheta", "ϑ");
    let s = s.replace("\\iota", "ι");
    let s = s.replace("\\kappa", "κ");
    let s = s.replace("\\lambda", "λ");
    let s = s.replace("\\mu", "μ");
    let s = s.replace("\\nu", "ν");
    let s = s.replace("\\xi", "ξ");
    let s = s.replace("\\omicron", "ο");
    let s = s.replace("\\pi", "π");
    let s = s.replace("\\varpi", "ϖ");
    let s = s.replace("\\rho", "ρ");
    let s = s.replace("\\varrho", "ϱ");
    let s = s.replace("\\sigma", "σ");
    let s = s.replace("\\varsigma", "ς");
    let s = s.replace("\\tau", "τ");
    let s = s.replace("\\upsilon", "υ");
    let s = s.replace("\\phi", "φ");
    let s = s.replace("\\varphi", "ϕ");
    let s = s.replace("\\chi", "χ");
    let s = s.replace("\\psi", "ψ");
    let s = s.replace("\\omega", "ω");
    
    // Accents and modifiers
    let re_bar = Regex::new(r"\\bar\{([^{}]*)\}").unwrap();
    let s = re_bar.replace_all(&s, "$1̄").to_string(); // combining overline
    let re_hat = Regex::new(r"\\hat\{([^{}]*)\}").unwrap();
    let s = re_hat.replace_all(&s, "$1̂").to_string(); // combining circumflex
    let re_vec = Regex::new(r"\\vec\{([^{}]*)\}").unwrap();
    let s = re_vec.replace_all(&s, "$1⃗").to_string(); // combining arrow
    let re_dot = Regex::new(r"\\dot\{([^{}]*)\}").unwrap();
    let s = re_dot.replace_all(&s, "$1̇").to_string(); // combining dot above
    let re_ddot = Regex::new(r"\\ddot\{([^{}]*)\}").unwrap();
    let s = re_ddot.replace_all(&s, "$1̈").to_string(); // combining diaeresis
    let re_tilde = Regex::new(r"\\tilde\{([^{}]*)\}").unwrap();
    let s = re_tilde.replace_all(&s, "$1̃").to_string(); // combining tilde
    let re_overline = Regex::new(r"\\overline\{([^{}]*)\}").unwrap();
    let s = re_overline.replace_all(&s, "$1̄").to_string();
    
    // Brackets
    let s = s.replace("\\left(", "(");
    let s = s.replace("\\right)", ")");
    let s = s.replace("\\left[", "[");
    let s = s.replace("\\right]", "]");
    let s = s.replace("\\left\\{", "{");
    let s = s.replace("\\right\\}", "}");
    let s = s.replace("\\left|", "|");
    let s = s.replace("\\right|", "|");
    let s = s.replace("\\langle", "⟨");
    let s = s.replace("\\rangle", "⟩");
    let s = s.replace("\\lfloor", "⌊");
    let s = s.replace("\\rfloor", "⌋");
    let s = s.replace("\\lceil", "⌈");
    let s = s.replace("\\rceil", "⌉");
    
    // Spacing and line breaks
    let s = s.replace("\\\\", "; ");
    let s = s.replace("\\quad", "  ");
    let s = s.replace("\\qquad", "    ");
    let s = s.replace("\\;", " ");
    let s = s.replace("\\:", " ");
    let s = s.replace("\\,", " ");
    let s = s.replace("\\ ", " ");
    let s = s.replace("\\!", "");
    
    // Text commands
    let re_text = Regex::new(r"\\text\{([^{}]*)\}").unwrap();
    let s = re_text.replace_all(&s, "$1").to_string();
    let re_mathrm = Regex::new(r"\\mathrm\{([^{}]*)\}").unwrap();
    let s = re_mathrm.replace_all(&s, "$1").to_string();
    let re_mathbf = Regex::new(r"\\mathbf\{([^{}]*)\}").unwrap();
    let s = re_mathbf.replace_all(&s, "$1").to_string();
    
    // Remove remaining unknown LaTeX commands
    let s = Regex::new(r"\\[a-zA-Z]+").unwrap().replace_all(&s, "").to_string();

    // Clean up remaining stray braces
    let s = s.replace('{', "(").replace('}', ")");

    // Collapse multiple spaces
    let s = Regex::new(r" {2,}").unwrap().replace_all(s.trim(), " ").to_string();

    s
}

/// Convert a string to Unicode superscript characters where possible
fn to_superscript(s: &str) -> String {
    s.chars().map(|c| match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴',
        '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', '9' => '⁹',
        '+' => '⁺', '-' => '⁻', '=' => '⁼', '(' => '⁽', ')' => '⁾',
        'n' => 'ⁿ', 'i' => 'ⁱ', 'x' => 'ˣ', 'y' => 'ʸ',
        'a' => 'ᵃ', 'b' => 'ᵇ', 'c' => 'ᶜ', 'd' => 'ᵈ', 'e' => 'ᵉ',
        'f' => 'ᶠ', 'g' => 'ᵍ', 'h' => 'ʰ', 'j' => 'ʲ', 'k' => 'ᵏ',
        'l' => 'ˡ', 'm' => 'ᵐ', 'o' => 'ᵒ', 'p' => 'ᵖ', 'r' => 'ʳ',
        's' => 'ˢ', 't' => 'ᵗ', 'u' => 'ᵘ', 'v' => 'ᵛ', 'w' => 'ʷ',
        'z' => 'ᶻ',
        _ => c,
    }).collect()
}

/// Convert a string to Unicode subscript characters where possible
fn to_subscript(s: &str) -> String {
    s.chars().map(|c| match c {
        '0' => '₀', '1' => '₁', '2' => '₂', '3' => '₃', '4' => '₄',
        '5' => '₅', '6' => '₆', '7' => '₇', '8' => '₈', '9' => '₉',
        '+' => '₊', '-' => '₋', '=' => '₌', '(' => '₍', ')' => '₎',
        'a' => 'ₐ', 'e' => 'ₑ', 'h' => 'ₕ', 'i' => 'ᵢ', 'j' => 'ⱼ',
        'k' => 'ₖ', 'l' => 'ₗ', 'm' => 'ₘ', 'n' => 'ₙ', 'o' => 'ₒ',
        'p' => 'ₚ', 'r' => 'ᵣ', 's' => 'ₛ', 't' => 'ₜ', 'u' => 'ᵤ',
        'v' => 'ᵥ', 'x' => 'ₓ',
        _ => c,
    }).collect()
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
            result.push(TextSpan { text: before.to_string(), bold, italic, underline, is_math: false, latex: None });
        }
        // Math span: keep original LaTeX for image rendering
        result.push(TextSpan { 
            text: latex_to_display(latex), 
            bold: false, 
            italic: false, 
            underline: false, 
            is_math: true, 
            latex: Some(latex.to_string()) 
        });
        last = full.end();
    }
    let after = &text[last..];
    if !after.is_empty() {
        result.push(TextSpan { text: after.to_string(), bold, italic, underline, is_math: false, latex: None });
    }
    if result.is_empty() { result.push(TextSpan { text: String::new(), bold, italic, underline, is_math: false, latex: None }); }
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
                        spans.push(TextSpan { 
                            text: rendered, 
                            bold: false, 
                            italic: false, 
                            underline: false, 
                            is_math: true, 
                            latex: Some(latex.to_string()) 
                        });
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

/// Decode base64 data URL to bytes
fn decode_data_url(url: &str) -> Option<Vec<u8>> {
    if !url.starts_with("data:") {
        return None;
    }
    let rest = url.strip_prefix("data:")?;
    let (_mime_part, data) = rest.split_once(',')?;
    
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.decode(data.trim()).ok()
}

fn parse_blocks(html: &str) -> Vec<Block> {
    let doc = Html::parse_fragment(html);
    let mut blocks = Vec::new();

    // We walk top-level block elements.
    // div[data-type="block-math"] is Tiptap's block-level math node.
    let body_sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, hr, table, div[data-type=\"block-math\"], img").unwrap();

    for el in doc.select(&body_sel) {
        let tag = el.value().name();
        match tag {
            "hr" => blocks.push(Block::Hr),
            "div" => {
                // Tiptap block math: <div data-type="block-math" data-latex="...">
                if let Some(latex) = el.value().attr("data-latex") {
                    let rendered = latex_to_display(latex);
                    blocks.push(Block::Para {
                        spans: vec![TextSpan { 
                            text: rendered, 
                            bold: false, 
                            italic: false, 
                            underline: false, 
                            is_math: true, 
                            latex: Some(latex.to_string()) 
                        }],
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
                    let mut spans = vec![TextSpan { text: prefix, bold: false, italic: false, underline: false, is_math: false, latex: None }];
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
                let mut rows = Vec::new();
                for (ri, tr) in el.select(&tr_sel).enumerate() {
                    let is_header = tr.select(&th_sel).next().is_some() || ri == 0;
                    let cells: Vec<String> = tr.select(&td_sel)
                        .map(|cell| cell.text().collect::<String>())
                        .collect();
                    if !cells.is_empty() {
                        rows.push(TableRowData { cells, is_header });
                    }
                }
                if !rows.is_empty() {
                    blocks.push(Block::Table { rows });
                }
            }
            "img" => {
                if let Some(src) = el.value().attr("src") {
                    let data = if src.starts_with("data:") {
                        decode_data_url(src)
                    } else if !src.starts_with("http") {
                        // Local file path
                        std::fs::read(src).ok()
                    } else {
                        None
                    };
                    
                    if let Some(bytes) = data {
                        // Try to get dimensions from attributes or style
                        let width = el.value().attr("width")
                            .and_then(|w| w.trim_end_matches("px").parse().ok())
                            .or_else(|| {
                                el.value().attr("style").and_then(|s| {
                                    s.split(';')
                                        .find(|p| p.contains("width"))
                                        .and_then(|p| p.split(':').nth(1))
                                        .and_then(|w| w.trim().trim_end_matches("px").parse().ok())
                                })
                            });
                        
                        blocks.push(Block::Image { data: bytes, width, height: None });
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
