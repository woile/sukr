//! Text escaping utilities for HTML and XML output.

/// Escape HTML special characters for safe embedding in HTML content.
///
/// Escapes: `&`, `<`, `>`, `"`, `'`
pub fn html_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    html_escape_into(&mut result, s);
    result
}

/// Escape HTML characters into an existing string.
///
/// This is more efficient when building output incrementally.
fn html_escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

/// Escape characters for safe embedding in code blocks.
///
/// Only escapes `&`, `<`, `>` — quotes are safe inside `<pre><code>`.
pub fn code_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    code_escape_into(&mut result, s);
    result
}

/// Escape code block characters into an existing string.
pub fn code_escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

/// Escape XML special characters for safe embedding in XML documents.
///
/// Escapes: `&`, `<`, `>`, `"`, `'`
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("Hello & World"), "Hello &amp; World");
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("it's"), "it&#39;s");
    }

    #[test]
    fn test_html_escape_into() {
        let mut buf = String::new();
        html_escape_into(&mut buf, "a < b");
        assert_eq!(buf, "a &lt; b");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("Hello & World"), "Hello &amp; World");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(xml_escape("it's"), "it&apos;s");
    }
}
