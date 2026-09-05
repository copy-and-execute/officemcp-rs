use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub enum MarkdownElement {
    Heading { level: usize, text: String, inlines: Vec<MarkdownInline> },
    Paragraph(Vec<MarkdownInline>),
    BulletList(Vec<Vec<MarkdownInline>>),
    NumberedList(Vec<Vec<MarkdownInline>>),
    CodeBlock { code: String, language: Option<String> },
    Blockquote(Vec<MarkdownInline>),
    HorizontalRule,
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Image { alt: String, url: String },
}

#[derive(Debug, Clone)]
pub enum MarkdownInline {
    Text(String),
    Bold(String),
    Italic(String),
    BoldItalic(String),
    Code(String),
    Strikethrough(String),
    Link { text: String, url: String },
    InlineImage { alt: String, url: String },
}

static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+)$").unwrap());
static HR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\*{3,}|-{3,}|_{3,})$").unwrap());
static BULLET_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*[-*+]\s+(.+)$").unwrap());
static NUMBERED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\d+\.\s+(.+)$").unwrap());
static TABLE_SEP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\|?\s*[-:]+[-|\s:]+\s*\|?\s*$").unwrap());
static IMAGE_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap());
static BOLD_ITALIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*\*(.+?)\*\*\*|___(.+?)___").unwrap());
static BOLD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap());
static ITALIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*([^*]+?)\*|_([^_]+?)_").unwrap());
static STRIKE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"~~(.+?)~~").unwrap());
static CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());
static LINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
static INLINE_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap());

pub fn parse(markdown: &str) -> Vec<MarkdownElement> {
    let mut elements = Vec::new();
    let normalized = markdown.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if HR_RE.is_match(trimmed) {
            elements.push(MarkdownElement::HorizontalRule);
            i += 1;
            continue;
        }
        if let Some(caps) = HEADING_RE.captures(trimmed) {
            let level = caps[1].len();
            let text = caps[2].trim().to_string();
            elements.push(MarkdownElement::Heading { level, text: text.clone(), inlines: parse_inlines(&text) });
            i += 1;
            continue;
        }
        if trimmed.starts_with("```") {
            let language = if trimmed.len() > 3 { Some(trimmed[3..].trim().to_string()).filter(|s| !s.is_empty()) } else { None };
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                code_lines.push(lines[i].to_string());
                i += 1;
            }
            i += 1;
            elements.push(MarkdownElement::CodeBlock { code: code_lines.join("\n"), language });
            continue;
        }
        if trimmed.contains('|') && i + 1 < lines.len() && TABLE_SEP_RE.is_match(lines[i+1]) {
            if let Some((table, new_i)) = parse_table(&lines, i) {
                elements.push(table);
                i = new_i;
                continue;
            }
        }
        if trimmed.starts_with('>') {
            let mut quote_lines = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                quote_lines.push(lines[i].trim_start().trim_start_matches('>').trim_start().to_string());
                i += 1;
            }
            let text = quote_lines.join(" ");
            elements.push(MarkdownElement::Blockquote(parse_inlines(&text)));
            continue;
        }
        if BULLET_RE.is_match(line) {
            let mut items = Vec::new();
            while i < lines.len() {
                if let Some(caps) = BULLET_RE.captures(lines[i]) {
                    items.push(parse_inlines(&caps[1]));
                    i += 1;
                } else { break; }
            }
            elements.push(MarkdownElement::BulletList(items));
            continue;
        }
        if NUMBERED_RE.is_match(line) {
            let mut items = Vec::new();
            while i < lines.len() {
                if let Some(caps) = NUMBERED_RE.captures(lines[i]) {
                    items.push(parse_inlines(&caps[1]));
                    i += 1;
                } else { break; }
            }
            elements.push(MarkdownElement::NumberedList(items));
            continue;
        }
        if let Some(caps) = IMAGE_BLOCK_RE.captures(trimmed) {
            if caps.get(0).map(|m| m.as_str()) == Some(trimmed) {
                elements.push(MarkdownElement::Image { alt: caps[1].to_string(), url: caps[2].to_string() });
                i += 1;
                continue;
            }
        }
        // paragraph collect
        let mut para_lines = Vec::new();
        while i < lines.len() {
            let cur = lines[i];
            if cur.trim().is_empty() ||
               HEADING_RE.is_match(cur.trim_start()) ||
               cur.trim_start().starts_with("```") ||
               cur.trim_start().starts_with('>') ||
               BULLET_RE.is_match(cur) ||
               NUMBERED_RE.is_match(cur) ||
               HR_RE.is_match(cur.trim_start()) {
                break;
            }
            para_lines.push(cur.to_string());
            i += 1;
        }
        if !para_lines.is_empty() {
            let text = para_lines.join(" ");
            elements.push(MarkdownElement::Paragraph(parse_inlines(&text)));
        }
    }
    elements
}

pub fn parse_inlines(text: &str) -> Vec<MarkdownInline> {
    let mut inlines = Vec::new();
    let mut remaining = text.to_string();
    while !remaining.is_empty() {
        // Inline image at start
        if let Some(caps) = INLINE_IMAGE_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                inlines.push(MarkdownInline::InlineImage { alt: caps[1].to_string(), url: caps[2].to_string() });
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // Link
        if let Some(caps) = LINK_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                // Ensure not preceded by ! (image)
                if remaining.starts_with('!') {
                    // treat as text
                } else {
                    inlines.push(MarkdownInline::Link { text: caps[1].to_string(), url: caps[2].to_string() });
                    remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                    continue;
                }
            }
        }
        // Code
        if let Some(caps) = CODE_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                inlines.push(MarkdownInline::Code(caps[1].to_string()));
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // BoldItalic
        if let Some(caps) = BOLD_ITALIC_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                let txt = caps.get(1).or(caps.get(2)).map(|m| m.as_str()).unwrap_or("");
                inlines.push(MarkdownInline::BoldItalic(txt.to_string()));
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // Bold
        if let Some(caps) = BOLD_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                let txt = caps.get(1).or(caps.get(2)).map(|m| m.as_str()).unwrap_or("");
                inlines.push(MarkdownInline::Bold(txt.to_string()));
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // Italic
        if let Some(caps) = ITALIC_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                let txt = caps.get(1).or(caps.get(2)).map(|m| m.as_str()).unwrap_or("");
                inlines.push(MarkdownInline::Italic(txt.to_string()));
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // Strikethrough
        if let Some(caps) = STRIKE_RE.captures(&remaining) {
            if caps.get(0).unwrap().start() == 0 {
                inlines.push(MarkdownInline::Strikethrough(caps[1].to_string()));
                remaining = remaining[caps.get(0).unwrap().end()..].to_string();
                continue;
            }
        }
        // Find next special char
        let next = find_next_special(&remaining);
        if next > 0 {
            let n = next as usize;
            inlines.push(MarkdownInline::Text(remaining[..n].to_string()));
            remaining = remaining[n..].to_string();
        } else if next == 0 {
            inlines.push(MarkdownInline::Text(remaining[..1].to_string()));
            remaining = remaining[1..].to_string();
        } else {
            inlines.push(MarkdownInline::Text(remaining.clone()));
            break;
        }
    }
    inlines
}

fn find_next_special(text: &str) -> i32 {
    let specials = ['*', '_', '`', '~', '[', '!'];
    let mut min = -1;
    for c in specials {
        if let Some(idx) = text.find(c) {
            if min == -1 || (idx as i32) < min {
                min = idx as i32;
            }
        }
    }
    min
}

fn parse_table(lines: &[&str], start: usize) -> Option<(MarkdownElement, usize)> {
    let headers = parse_table_row(lines[start]);
    if headers.is_empty() { return None; }
    let header_count = headers.len();
    let mut rows = Vec::new();
    let mut i = start + 2;
    while i < lines.len() {
        let line = lines[i].trim();
        if !line.contains('|') { break; }
        let mut row = parse_table_row(line);
        if row.is_empty() { i+=1; continue; }
        while row.len() < header_count { row.push(String::new()); }
        if row.len() > header_count { row.truncate(header_count); }
        rows.push(row);
        i+=1;
    }
    Some((MarkdownElement::Table { headers, rows }, i))
}

fn parse_table_row(line: &str) -> Vec<String> {
    const PLACEHOLDER: &str = "\x00PIPE\x00";
    let processed = line.replace("\\|", PLACEHOLDER);
    let mut cells: Vec<String> = processed.split('|').map(|c| c.trim().replace(PLACEHOLDER, "|")).collect();
    if !cells.is_empty() && cells[0].is_empty() { cells.remove(0); }
    if !cells.is_empty() && cells.last().map(|s| s.is_empty()).unwrap_or(false) { cells.pop(); }
    cells
}
