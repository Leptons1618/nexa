//! Rich content renderer — renders markdown, code blocks, LaTeX, tables,
//! and HTML content inside chat bubbles.
//!
//! Rendering strategy:
//! 1. Parse block-level elements (code fences, LaTeX, headers, lists, tables, etc.)
//! 2. Process inline markup (bold, italic, code, links, images, LaTeX)
//! 3. Inject processed HTML via `dangerous_inner_html`
//! 4. Trigger highlight.js + KaTeX rendering via JS after mount

use dioxus::prelude::*;

/// Render rich text content (markdown/code/LaTeX/HTML) as safe HTML.
#[component]
pub fn RichContent(text: String, streaming: Option<bool>) -> Element {
    let html = render_markdown_to_html(&text);
    let is_streaming = streaming.unwrap_or(false);

    // Re-trigger syntax highlighting and KaTeX after every render
    use_effect(move || {
        trigger_rendering();
    });

    rsx! {
        div {
            class: if is_streaming { "rich-content rich-content--streaming" } else { "rich-content" },
            dangerous_inner_html: "{html}",
        }
    }
}

/// Call highlight.js and KaTeX on newly rendered content.
fn trigger_rendering() {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window"], js_name = "requestAnimationFrame")]
        fn request_animation_frame(closure: &Closure<dyn FnMut()>);
    }

    let cb = Closure::once(move || {
        if let Some(window) = web_sys::window() {
            if let Some(_document) = window.document() {
                // highlight.js — apply to un-highlighted code blocks
                let _ = js_sys::eval(
                    r#"
                    if (typeof hljs !== 'undefined') {
                        document.querySelectorAll('.code-block code:not(.hljs)').forEach(function(el) {
                            hljs.highlightElement(el);
                        });
                    }
                    "#,
                );
                // KaTeX — render block and inline math
                let _ = js_sys::eval(
                    r#"
                    if (typeof katex !== 'undefined') {
                        document.querySelectorAll('.katex-block:not(.katex-rendered)').forEach(function(el) {
                            try {
                                var latex = el.getAttribute('data-latex');
                                if (latex) {
                                    katex.render(latex, el, { displayMode: true, throwOnError: false });
                                    el.classList.add('katex-rendered');
                                }
                            } catch(e) {}
                        });
                        document.querySelectorAll('.katex-inline:not(.katex-rendered)').forEach(function(el) {
                            try {
                                var latex = el.getAttribute('data-latex');
                                if (latex) {
                                    katex.render(latex, el, { displayMode: false, throwOnError: false });
                                    el.classList.add('katex-rendered');
                                }
                            } catch(e) {}
                        });
                    }
                    "#,
                );
            }
        }
    });
    request_animation_frame(&cb);
    cb.forget(); // intentional: one-shot callback
}

/// Lightweight Markdown→HTML converter.
///
/// Supports:
/// - Fenced code blocks (```lang ... ```)
/// - Inline code (`...`)
/// - Bold (**...**), Italic (*...*), Strikethrough (~~...~~)
/// - Bold+Italic (***...***) 
/// - Headers (# ## ### etc.)
/// - Ordered and unordered lists (with nesting)
/// - Task lists (- [ ] / - [x])
/// - Blockquotes (> ...)
/// - Horizontal rules (--- or ***)
/// - Links [text](url), Images ![alt](url)
/// - Tables (| col | col |)
/// - LaTeX blocks ($$...$$), Inline LaTeX ($...$)
/// - Line breaks (trailing `\` or double space)
/// - Superscript (^...^)
/// - HTML passthrough
fn render_markdown_to_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2);
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut in_list = false;
    let mut list_type: &str = "";

    while i < lines.len() {
        let line = lines[i];

        // ── Fenced code blocks ───────────────────────────
        if line.trim_start().starts_with("```") {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            let lang = line.trim_start().trim_start_matches('`').trim();
            let lang_attr = if lang.is_empty() {
                String::new()
            } else {
                format!(" class=\"language-{lang}\"")
            };
            let lang_label = if lang.is_empty() {
                "<div class=\"code-block-header\"><span class=\"code-lang\">code</span>\
                 <button class=\"code-copy-btn\" onclick=\"navigator.clipboard.writeText(this.parentElement.nextElementSibling.textContent)\">Copy</button></div>"
                    .to_string()
            } else {
                format!(
                    "<div class=\"code-block-header\"><span class=\"code-lang\">{}</span>\
                     <button class=\"code-copy-btn\" onclick=\"navigator.clipboard.writeText(this.parentElement.nextElementSibling.textContent)\">Copy</button></div>",
                    escape_html(lang)
                )
            };
            result.push_str(&format!(
                "<div class=\"code-block\">{lang_label}<pre><code{lang_attr}>"
            ));
            i += 1;
            let mut first_line = true;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                if !first_line {
                    result.push('\n');
                }
                result.push_str(&escape_html(lines[i]));
                first_line = false;
                i += 1;
            }
            result.push_str("</code></pre></div>");
            i += 1;
            continue;
        }

        // ── LaTeX display block ($$...$$) ────────────────
        if line.trim().starts_with("$$") {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            let mut latex = String::new();
            if line.trim() == "$$" {
                i += 1;
                while i < lines.len() && lines[i].trim() != "$$" {
                    if !latex.is_empty() {
                        latex.push('\n');
                    }
                    latex.push_str(lines[i]);
                    i += 1;
                }
                i += 1;
            } else {
                let content = line.trim().trim_start_matches("$$").trim_end_matches("$$");
                latex = content.to_string();
                i += 1;
            }
            result.push_str(&format!(
                "<div class=\"katex-block\" data-latex=\"{}\">{}</div>",
                escape_html_attr(&latex),
                escape_html(&latex)
            ));
            continue;
        }

        // ── LaTeX display block (\[...\]) ────────────────
        if line.trim().starts_with("\\[") {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            let mut latex = String::new();
            if line.trim() == "\\[" {
                i += 1;
                while i < lines.len() && lines[i].trim() != "\\]" {
                    if !latex.is_empty() {
                        latex.push('\n');
                    }
                    latex.push_str(lines[i]);
                    i += 1;
                }
                i += 1;
            } else {
                let trimmed = line.trim();
                let content = trimmed
                    .strip_prefix("\\[")
                    .unwrap_or(trimmed)
                    .strip_suffix("\\]")
                    .unwrap_or(trimmed)
                    .trim();
                latex = content.to_string();
                i += 1;
            }
            result.push_str(&format!(
                "<div class=\"katex-block\" data-latex=\"{}\">{}</div>",
                escape_html_attr(&latex),
                escape_html(&latex)
            ));
            continue;
        }

        // ── Table detection ──────────────────────────────
        if line.contains('|') && line.trim().starts_with('|') && line.trim().ends_with('|') {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            let table_html = parse_table(&lines, &mut i);
            result.push_str(&table_html);
            continue;
        }

        // Close list if current line is not a list item and not empty
        if in_list && !is_list_item(line) && !line.trim().is_empty() {
            result.push_str(&format!("</{list_type}>"));
            in_list = false;
        }

        // ── Horizontal rule ──────────────────────────────
        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            result.push_str("<hr>");
            i += 1;
            continue;
        }

        // ── Headers ──────────────────────────────────────
        if let Some(header) = parse_header(line) {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            result.push_str(&header);
            i += 1;
            continue;
        }

        // ── Blockquote ──────────────────────────────────
        if line.trim_start().starts_with("> ") || line.trim_start() == ">" {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            result.push_str("<blockquote>");
            while i < lines.len()
                && (lines[i].trim_start().starts_with("> ") || lines[i].trim_start() == ">")
            {
                let content = lines[i]
                    .trim_start()
                    .strip_prefix("> ")
                    .or_else(|| lines[i].trim_start().strip_prefix(">"))
                    .unwrap_or("");
                result.push_str(&format!("<p>{}</p>", process_inline(content)));
                i += 1;
            }
            result.push_str("</blockquote>");
            continue;
        }

        // ── Task list items ──────────────────────────────
        if is_task_list_item(line) {
            if !in_list || list_type != "ul" {
                if in_list {
                    result.push_str(&format!("</{list_type}>"));
                }
                result.push_str("<ul class=\"task-list\">");
                in_list = true;
                list_type = "ul";
            }
            let trimmed = line.trim_start();
            let (checked, content) = parse_task_item(trimmed);
            if checked {
                result.push_str("<li class=\"task-list-item task-list-item--checked\">");
                result.push_str("<span class=\"task-checkbox task-checkbox--checked\">&#9745;</span> ");
            } else {
                result.push_str("<li class=\"task-list-item\">");
                result.push_str("<span class=\"task-checkbox\">&#9744;</span> ");
            }
            result.push_str(&process_inline(content));
            result.push_str("</li>");
            i += 1;
            continue;
        }

        // ── Unordered list items ─────────────────────────
        if line.trim_start().starts_with("- ")
            || line.trim_start().starts_with("* ")
            || line.trim_start().starts_with("+ ")
        {
            if !in_list || list_type != "ul" {
                if in_list {
                    result.push_str(&format!("</{list_type}>"));
                }
                result.push_str("<ul>");
                in_list = true;
                list_type = "ul";
            }
            let content = &line.trim_start()[2..];
            result.push_str("<li>");
            result.push_str(&process_inline(content));
            result.push_str("</li>");
            i += 1;
            continue;
        }

        // ── Ordered list items ───────────────────────────
        if let Some(content) = parse_ordered_list_item(line) {
            if !in_list || list_type != "ol" {
                if in_list {
                    result.push_str(&format!("</{list_type}>"));
                }
                result.push_str("<ol>");
                in_list = true;
                list_type = "ol";
            }
            result.push_str("<li>");
            result.push_str(&process_inline(content));
            result.push_str("</li>");
            i += 1;
            continue;
        }

        // ── Empty line ──────────────────────────────────
        if line.trim().is_empty() {
            if in_list {
                result.push_str(&format!("</{list_type}>"));
                in_list = false;
            }
            i += 1;
            continue;
        }

        // ── HTML passthrough ────────────────────────────
        if line.trim_start().starts_with('<')
            && (line.contains("<table")
                || line.contains("<div")
                || line.contains("<img")
                || line.contains("<br")
                || line.contains("<hr"))
        {
            result.push_str(line);
            i += 1;
            continue;
        }

        // ── Normal paragraph (collect consecutive lines) ─
        let mut para = String::new();
        while i < lines.len() {
            let l = lines[i];
            if l.trim().is_empty()
                || l.trim_start().starts_with('#')
                || l.trim_start().starts_with("```")
                || l.trim_start().starts_with("> ")
                || l.trim_start().starts_with("- ")
                || l.trim_start().starts_with("* ")
                || l.trim_start().starts_with("+ ")
                || l.trim().starts_with("$$")
                || l.trim().starts_with("\\[")
                || is_task_list_item(l)
                || (l.contains('|') && l.trim().starts_with('|') && l.trim().ends_with('|'))
                || l.trim() == "---"
                || l.trim() == "***"
                || l.trim() == "___"
                || parse_ordered_list_item(l).is_some()
            {
                break;
            }
            if !para.is_empty() {
                // Trailing backslash or double space → <br>
                if para.ends_with('\\') {
                    para.pop();
                    para.push_str("<br>");
                } else if para.ends_with("  ") {
                    let trimmed = para.trim_end().to_string();
                    para = trimmed;
                    para.push_str("<br>");
                } else {
                    para.push(' ');
                }
            }
            para.push_str(l);
            i += 1;
        }
        if !para.is_empty() {
            result.push_str("<p>");
            result.push_str(&process_inline(&para));
            result.push_str("</p>");
        }
    }

    if in_list {
        result.push_str(&format!("</{list_type}>"));
    }

    result
}

// ── Task list helpers ────────────────────────────────────

fn is_task_list_item(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("- [ ] ") || t.starts_with("- [x] ") || t.starts_with("- [X] ")
        || t.starts_with("* [ ] ") || t.starts_with("* [x] ") || t.starts_with("* [X] ")
}

fn parse_task_item(trimmed: &str) -> (bool, &str) {
    if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ")
        || trimmed.starts_with("* [x] ") || trimmed.starts_with("* [X] ")
    {
        (true, &trimmed[6..])
    } else if trimmed.starts_with("- [ ] ") || trimmed.starts_with("* [ ] ") {
        (false, &trimmed[6..])
    } else {
        (false, trimmed)
    }
}

/// Parse a markdown table starting at the current line.
fn parse_table(lines: &[&str], i: &mut usize) -> String {
    let mut html = String::from("<div class=\"table-wrapper\"><table>");

    // Header row
    if *i < lines.len() {
        let cells = parse_table_row(lines[*i]);
        html.push_str("<thead><tr>");
        for cell in &cells {
            html.push_str(&format!("<th>{}</th>", process_inline(cell)));
        }
        html.push_str("</tr></thead>");
        *i += 1;
    }

    // Separator row (skip it)
    if *i < lines.len() && lines[*i].contains("---") {
        *i += 1;
    }

    // Data rows
    html.push_str("<tbody>");
    while *i < lines.len() && lines[*i].contains('|') && lines[*i].trim().starts_with('|') {
        let cells = parse_table_row(lines[*i]);
        html.push_str("<tr>");
        for cell in &cells {
            html.push_str(&format!("<td>{}</td>", process_inline(cell)));
        }
        html.push_str("</tr>");
        *i += 1;
    }
    html.push_str("</tbody></table></div>");
    html
}

fn parse_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

fn is_list_item(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("- ")
        || t.starts_with("* ")
        || t.starts_with("+ ")
        || is_task_list_item(line)
        || parse_ordered_list_item(line).is_some()
}

fn parse_ordered_list_item(line: &str) -> Option<&str> {
    let t = line.trim_start();
    let dot_pos = t.find(". ")?;
    if dot_pos > 0 && dot_pos <= 3 && t[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
        Some(&t[dot_pos + 2..])
    } else {
        None
    }
}

fn parse_header(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level >= 1 && level <= 6 && trimmed.len() > level && trimmed.as_bytes()[level] == b' ' {
        let content = &trimmed[level + 1..];
        Some(format!(
            "<h{level}>{}</h{level}>",
            process_inline(content)
        ))
    } else {
        None
    }
}

/// Process inline markup: bold, italic, code, links, images, inline LaTeX, line breaks.
fn process_inline(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // HTML <br> passthrough
        if i + 3 < len && chars[i] == '<' && chars[i + 1] == 'b' && chars[i + 2] == 'r' && chars[i + 3] == '>' {
            result.push_str("<br>");
            i += 4;
            continue;
        }

        // Inline code
        if chars[i] == '`' {
            if let Some(end) = find_closing(&chars, i + 1, '`') {
                let code: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("<code>{}</code>", escape_html(&code)));
                i = end + 1;
                continue;
            }
        }

        // Inline LaTeX \(...\)
        if chars[i] == '\\' && i + 1 < len && chars[i + 1] == '(' {
            let start_pos = i + 2;
            let mut found = false;
            let mut end_pos = start_pos;
            while end_pos + 1 < len {
                if chars[end_pos] == '\\' && chars[end_pos + 1] == ')' {
                    found = true;
                    break;
                }
                end_pos += 1;
            }
            if found {
                let latex: String = chars[start_pos..end_pos].iter().collect();
                result.push_str(&format!(
                    "<span class=\"katex-inline\" data-latex=\"{}\">{}</span>",
                    escape_html_attr(&latex),
                    escape_html(&latex)
                ));
                i = end_pos + 2;
                continue;
            }
        }

        // Inline LaTeX $...$ (not $$)
        if chars[i] == '$' && (i + 1 < len && chars[i + 1] != '$') {
            if let Some(end) = find_closing(&chars, i + 1, '$') {
                if end > i + 1 {
                    let latex: String = chars[i + 1..end].iter().collect();
                    result.push_str(&format!(
                        "<span class=\"katex-inline\" data-latex=\"{}\">{}</span>",
                        escape_html_attr(&latex),
                        escape_html(&latex)
                    ));
                    i = end + 1;
                    continue;
                }
            }
        }

        // Bold + italic ***...***
        if i + 2 < len && chars[i] == '*' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            if let Some(end) = find_triple_closing(&chars, i + 3, '*') {
                let inner: String = chars[i + 3..end].iter().collect();
                result.push_str(&format!("<strong><em>{}</em></strong>", process_inline(&inner)));
                i = end + 3;
                continue;
            }
        }

        // Bold **...**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_closing(&chars, i + 2, '*') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<strong>{}</strong>", process_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Bold __...__
        if i + 1 < len && chars[i] == '_' && chars[i + 1] == '_' {
            if let Some(end) = find_double_closing(&chars, i + 2, '_') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<strong>{}</strong>", process_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Italic *...*
        if chars[i] == '*' && (i + 1 < len && chars[i + 1] != '*' && chars[i + 1] != ' ') {
            if let Some(end) = find_closing(&chars, i + 1, '*') {
                if end > i + 1 {
                    let inner: String = chars[i + 1..end].iter().collect();
                    result.push_str(&format!("<em>{}</em>", process_inline(&inner)));
                    i = end + 1;
                    continue;
                }
            }
        }

        // Italic _..._
        if chars[i] == '_' && (i + 1 < len && chars[i + 1] != '_' && chars[i + 1] != ' ') {
            if let Some(end) = find_closing(&chars, i + 1, '_') {
                if end > i + 1 {
                    let inner: String = chars[i + 1..end].iter().collect();
                    result.push_str(&format!("<em>{}</em>", process_inline(&inner)));
                    i = end + 1;
                    continue;
                }
            }
        }

        // Image ![alt](url)
        if chars[i] == '!' && i + 1 < len && chars[i + 1] == '[' {
            if let Some((alt, url, end)) = parse_link_or_image(&chars, i + 1) {
                result.push_str(&format!(
                    "<img src=\"{}\" alt=\"{}\" class=\"chat-image\" loading=\"lazy\">",
                    escape_html_attr(&url),
                    escape_html(&alt)
                ));
                i = end;
                continue;
            }
        }

        // Link [text](url)
        if chars[i] == '[' {
            if let Some((link_text, url, end)) = parse_link_or_image(&chars, i) {
                result.push_str(&format!(
                    "<a href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a>",
                    escape_html_attr(&url),
                    process_inline(&link_text)
                ));
                i = end;
                continue;
            }
        }

        // Strikethrough ~~...~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_double_closing(&chars, i + 2, '~') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<del>{}</del>", process_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Superscript ^...^
        if chars[i] == '^' && i + 1 < len && chars[i + 1] != ' ' {
            if let Some(end) = find_closing(&chars, i + 1, '^') {
                if end > i + 1 {
                    let inner: String = chars[i + 1..end].iter().collect();
                    result.push_str(&format!("<sup>{}</sup>", escape_html(&inner)));
                    i = end + 1;
                    continue;
                }
            }
        }

        // Normal character
        result.push_str(&escape_html_char(chars[i]));
        i += 1;
    }

    result
}

fn find_closing(chars: &[char], start: usize, delim: char) -> Option<usize> {
    for j in start..chars.len() {
        if chars[j] == delim {
            return Some(j);
        }
    }
    None
}

fn find_double_closing(chars: &[char], start: usize, delim: char) -> Option<usize> {
    for j in start..chars.len().saturating_sub(1) {
        if chars[j] == delim && chars[j + 1] == delim {
            return Some(j);
        }
    }
    None
}

fn find_triple_closing(chars: &[char], start: usize, delim: char) -> Option<usize> {
    for j in start..chars.len().saturating_sub(2) {
        if chars[j] == delim && chars[j + 1] == delim && chars[j + 2] == delim {
            return Some(j);
        }
    }
    None
}

fn parse_link_or_image(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    if chars[start] != '[' {
        return None;
    }
    let close_bracket = find_closing(chars, start + 1, ']')?;
    let text: String = chars[start + 1..close_bracket].iter().collect();
    if close_bracket + 1 >= chars.len() || chars[close_bracket + 1] != '(' {
        return None;
    }
    let close_paren = find_closing(chars, close_bracket + 2, ')')?;
    let url: String = chars[close_bracket + 2..close_paren].iter().collect();
    Some((text, url, close_paren + 1))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_html_attr(s: &str) -> String {
    escape_html(s).replace('\'', "&#39;")
}

fn escape_html_char(c: char) -> String {
    match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        _ => c.to_string(),
    }
}
