//! Simple Discord-style markdown: **bold**, *italic*, `code`, ||spoiler||

/// Convert Discord markdown to HTML. Escapes HTML, then applies formatting.
pub fn discord_markdown_to_html(s: &str) -> String {
    let escaped = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    apply_markdown(&escaped)
}

fn apply_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    let mut i = 0;
    
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("***") {
            if let Some(pos) = rest[3..].find("***") {
                out.push_str("<strong><em>");
                out.push_str(&apply_markdown(&rest[3..3 + pos]));
                out.push_str("</em></strong>");
                i += 3 + pos + 3;
                continue;
            }
        }
        if rest.starts_with("**") {
            if let Some(pos) = rest[2..].find("**") {
                out.push_str("<strong style=\"font-weight:600;\">");
                out.push_str(&apply_markdown(&rest[2..2 + pos]));
                out.push_str("</strong>");
                i += 2 + pos + 2;
                continue;
            }
        }
        if rest.starts_with('*') && !rest.starts_with("**") {
            if let Some(pos) = rest[1..].find('*') {
                out.push_str("<em>");
                out.push_str(&html_escape(&rest[1..1 + pos]));
                out.push_str("</em>");
                i += 1 + pos + 1;
                continue;
            }
        }
        if rest.starts_with('`') {
            if let Some(pos) = rest[1..].find('`') {
                out.push_str("<code style=\"background:rgba(255,255,255,0.1);padding:0.1em 0.3em;border-radius:4px;font-size:0.9em;\">");
                out.push_str(&html_escape(&rest[1..1 + pos]));
                out.push_str("</code>");
                i += 1 + pos + 1;
                continue;
            }
        }
        if rest.starts_with("||") {
            if let Some(pos) = rest[2..].find("||") {
                out.push_str("<span style=\"background:#1a1a1a;color:#1a1a1a;border-radius:2px;\" class=\"spoiler\" title=\"Spoiler\">");
                out.push_str(&html_escape(&rest[2..2 + pos]));
                out.push_str("</span>");
                i += 2 + pos + 2;
                continue;
            }
        }
        if let Some(c) = rest.chars().next() {
            out.push_str(&html_escape_char(c));
            i += c.len_utf8();
        }
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_escape_char(c: char) -> String {
    match c {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        '"' => "&quot;".to_string(),
        _ => c.to_string(),
    }
}

