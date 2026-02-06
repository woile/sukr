//! Markdown to HTML rendering via pulldown-cmark with syntax highlighting.

use crate::escape::{code_escape, html_escape};
use crate::highlight::{Language, highlight_code};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Serialize;

/// A heading anchor extracted from markdown content.
#[derive(Debug, Clone, Serialize)]
pub struct Anchor {
    /// Heading ID attribute (slug)
    pub id: String,
    /// Heading text content
    pub label: String,
    /// Heading level (2-6, h1 excluded)
    pub level: u8,
}

/// Render markdown content to HTML with syntax highlighting.
/// Returns the HTML output and a list of extracted heading anchors.
pub fn markdown_to_html(markdown: &str) -> (String, Vec<Anchor>) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH;

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    let mut anchors = Vec::new();
    let mut code_block_lang: Option<String> = None;
    let mut code_block_content = String::new();
    let mut in_code_block = false;

    // Image alt text accumulation state
    let mut image_alt_content: Option<String> = None;
    let mut image_attrs: Option<(String, String)> = None; // (src, title)

    // Heading accumulation state
    let mut heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                // Extract language from code fence
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang_str = lang.as_ref().split_whitespace().next().unwrap_or("");
                        if lang_str.is_empty() {
                            None
                        } else {
                            Some(lang_str.to_string())
                        }
                    },
                    CodeBlockKind::Indented => None,
                };
                in_code_block = true;
                code_block_content.clear();
            },
            Event::Text(text) if in_code_block => {
                // Accumulate code block content
                code_block_content.push_str(&text);
            },
            Event::Text(text) if image_alt_content.is_some() => {
                // Accumulate image alt text
                if let Some(ref mut alt) = image_alt_content {
                    alt.push_str(&text);
                }
            },
            Event::End(TagEnd::CodeBlock) => {
                // Render the code block with highlighting
                let lang_str = code_block_lang.as_deref().unwrap_or("");

                // Mermaid diagrams: render to SVG
                if lang_str == "mermaid" {
                    match crate::mermaid::render_diagram(&code_block_content) {
                        Ok(svg) => {
                            html_output.push_str("<div class=\"mermaid-diagram\">\n");
                            html_output.push_str(&svg);
                            html_output.push_str("\n</div>\n");
                        },
                        Err(e) => {
                            eprintln!("mermaid render error: {e}");
                            html_output.push_str("<pre class=\"mermaid-error\"><code>");
                            html_output.push_str(&html_escape(&code_block_content));
                            html_output.push_str("</code></pre>\n");
                        },
                    }
                } else {
                    // Code blocks: syntax highlighting
                    html_output.push_str("<pre><code");

                    if let Some(lang) = Language::from_fence(lang_str) {
                        // Supported language: apply tree-sitter highlighting
                        html_output.push_str(&format!(" class=\"language-{}\">", lang_str));
                        html_output.push_str(&highlight_code(lang, &code_block_content));
                    } else {
                        // Unsupported language: render as plain escaped text
                        if !lang_str.is_empty() {
                            html_output.push_str(&format!(" class=\"language-{}\">", lang_str));
                        } else {
                            html_output.push('>');
                        }
                        html_output.push_str(&code_escape(&code_block_content));
                    }

                    html_output.push_str("</code></pre>\n");
                }

                code_block_lang = None;
                in_code_block = false;
                code_block_content.clear();
            },
            Event::Text(text) if heading_level.is_some() => {
                // Accumulate heading text
                heading_text.push_str(&text);
                html_output.push_str(&html_escape(&text));
            },
            Event::Text(text) => {
                // Regular text outside code blocks
                html_output.push_str(&html_escape(&text));
            },
            Event::Code(text) => {
                // Inline code
                html_output.push_str("<code>");
                html_output.push_str(&html_escape(&text));
                html_output.push_str("</code>");
            },
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                // Begin accumulating alt text; defer rendering to End event
                image_alt_content = Some(String::new());
                image_attrs = Some((dest_url.to_string(), title.to_string()));
            },
            Event::Start(Tag::Heading { level, .. }) => {
                // Begin accumulating heading text
                heading_level = Some(level);
                heading_text.clear();
                let level_num = level as u8;
                html_output.push_str(&format!("<h{}", level_num));
                // ID will be added at End event after we have the text
            },
            Event::Start(tag) => {
                html_output.push_str(&start_tag_to_html(&tag));
            },
            Event::End(TagEnd::Image) => {
                // Render image with accumulated alt text
                let alt = image_alt_content.take().unwrap_or_default();
                if let Some((src, title)) = image_attrs.take() {
                    if title.is_empty() {
                        html_output.push_str(&format!(
                            "<img src=\"{}\" alt=\"{}\" />",
                            html_escape(&src),
                            html_escape(&alt)
                        ));
                    } else {
                        html_output.push_str(&format!(
                            "<img src=\"{}\" alt=\"{}\" title=\"{}\" />",
                            html_escape(&src),
                            html_escape(&alt),
                            html_escape(&title)
                        ));
                    }
                }
            },
            Event::End(TagEnd::Heading(level)) => {
                // Generate slug ID from heading text
                let id = slugify(&heading_text);
                let level_num = level as u8;

                // We need to go back and insert the id attribute and close the tag
                // The heading was opened as "<hN" - find it and complete with id and >
                if let Some(pos) = html_output.rfind(&format!("<h{}", level_num)) {
                    let insert_pos = pos + format!("<h{}", level_num).len();
                    html_output.insert_str(insert_pos, &format!(" id=\"{}\">", id));
                }
                // Add pilcrow anchor link for deep-linking (hover-reveal via CSS)
                html_output.push_str(&format!(
                    "<a class=\"heading-anchor\" href=\"#{}\">¶</a></h{}>\n",
                    id, level_num
                ));

                // Extract anchor for h2-h6 (skip h1)
                if level_num >= 2 {
                    anchors.push(Anchor {
                        id,
                        label: heading_text.clone(),
                        level: level_num,
                    });
                }

                heading_level = None;
                heading_text.clear();
            },
            Event::End(tag) => {
                html_output.push_str(&end_tag_to_html(&tag));
            },
            Event::SoftBreak => {
                html_output.push('\n');
            },
            Event::HardBreak => {
                html_output.push_str("<br />\n");
            },
            Event::Rule => {
                html_output.push_str("<hr />\n");
            },
            Event::Html(html) | Event::InlineHtml(html) => {
                html_output.push_str(&html);
            },
            Event::FootnoteReference(name) => {
                html_output.push_str(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{}\">{}</a></sup>",
                    name, name
                ));
            },
            Event::TaskListMarker(checked) => {
                let checkbox = if checked {
                    "<input type=\"checkbox\" checked disabled />"
                } else {
                    "<input type=\"checkbox\" disabled />"
                };
                html_output.push_str(checkbox);
            },
            Event::InlineMath(latex) => match crate::math::render_math(&latex, false) {
                Ok(rendered) => html_output.push_str(&rendered),
                Err(e) => {
                    eprintln!("math render error: {e}");
                    html_output.push_str("<code class=\"math-error\">");
                    html_output.push_str(&html_escape(&latex));
                    html_output.push_str("</code>");
                },
            },
            Event::DisplayMath(latex) => match crate::math::render_math(&latex, true) {
                Ok(rendered) => {
                    html_output.push_str("<div class=\"math-display\">\n");
                    html_output.push_str(&rendered);
                    html_output.push_str("\n</div>\n");
                },
                Err(e) => {
                    eprintln!("math render error: {e}");
                    html_output.push_str("<pre class=\"math-error\">");
                    html_output.push_str(&html_escape(&latex));
                    html_output.push_str("</pre>\n");
                },
            },
        }
    }

    (html_output, anchors)
}

/// Convert heading text to a URL-friendly slug ID.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn start_tag_to_html(tag: &Tag) -> String {
    match tag {
        Tag::Paragraph => "<p>".to_string(),
        Tag::Heading { level, .. } => format!("<h{}>", *level as u8),
        Tag::BlockQuote(_) => "<blockquote>\n".to_string(),
        Tag::CodeBlock(_) => String::new(), // Handled separately
        Tag::List(Some(start)) => format!("<ol start=\"{}\">\n", start),
        Tag::List(None) => "<ul>\n".to_string(),
        Tag::Item => "<li>".to_string(),
        Tag::FootnoteDefinition(name) => {
            format!("<div class=\"footnote\" id=\"fn-{}\">", name)
        },
        Tag::Table(_) => "<table>\n".to_string(),
        Tag::TableHead => "<thead>\n<tr>\n".to_string(),
        Tag::TableRow => "<tr>\n".to_string(),
        Tag::TableCell => "<td>".to_string(),
        Tag::Emphasis => "<em>".to_string(),
        Tag::Strong => "<strong>".to_string(),
        Tag::Strikethrough => "<del>".to_string(),
        Tag::Link {
            dest_url, title, ..
        } => {
            if title.is_empty() {
                format!("<a href=\"{}\">", html_escape(dest_url))
            } else {
                format!(
                    "<a href=\"{}\" title=\"{}\">",
                    html_escape(dest_url),
                    html_escape(title)
                )
            }
        },
        Tag::Image { .. } => String::new(), // Handled separately in main loop
        Tag::HtmlBlock => String::new(),
        Tag::MetadataBlock(_) => String::new(),
        Tag::DefinitionListTitle => "<dt>".to_string(),
        Tag::DefinitionListDefinition => "<dd>".to_string(),
        Tag::DefinitionList => "<dl>".to_string(),
    }
}

fn end_tag_to_html(tag: &TagEnd) -> String {
    match tag {
        TagEnd::Paragraph => "</p>\n".to_string(),
        TagEnd::Heading(level) => format!("</h{}>\n", *level as u8),
        TagEnd::BlockQuote(_) => "</blockquote>\n".to_string(),
        TagEnd::CodeBlock => String::new(), // Handled separately
        TagEnd::List(ordered) => {
            if *ordered {
                "</ol>\n".to_string()
            } else {
                "</ul>\n".to_string()
            }
        },
        TagEnd::Item => "</li>\n".to_string(),
        TagEnd::FootnoteDefinition => "</div>\n".to_string(),
        TagEnd::Table => "</table>\n".to_string(),
        TagEnd::TableHead => "</tr>\n</thead>\n".to_string(),
        TagEnd::TableRow => "</tr>\n".to_string(),
        TagEnd::TableCell => "</td>\n".to_string(),
        TagEnd::Emphasis => "</em>".to_string(),
        TagEnd::Strong => "</strong>".to_string(),
        TagEnd::Strikethrough => "</del>".to_string(),
        TagEnd::Link => "</a>".to_string(),
        TagEnd::Image => String::new(), // Handled separately in main loop
        TagEnd::HtmlBlock => String::new(),
        TagEnd::MetadataBlock(_) => String::new(),
        TagEnd::DefinitionListTitle => "</dt>\n".to_string(),
        TagEnd::DefinitionListDefinition => "</dd>\n".to_string(),
        TagEnd::DefinitionList => "</dl>\n".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let md = "# Hello\n\nThis is a *test*.";
        let (html, _) = markdown_to_html(md);
        // Heading includes pilcrow anchor for deep-linking
        assert!(html.contains(
            "<h1 id=\"hello\">Hello<a class=\"heading-anchor\" href=\"#hello\">¶</a></h1>"
        ));
        assert!(html.contains("<em>test</em>"));
    }

    #[test]
    fn test_code_block_highlighting() {
        let md = "```rust\nfn main() {}\n```";
        let (html, _) = markdown_to_html(md);

        // Should contain highlighted code
        assert!(html.contains("<pre><code"));
        assert!(html.contains("language-rust"));
        assert!(html.contains("class=\"hl-"));
    }

    #[test]
    fn test_code_block_unknown_language() {
        let md = "```unknown\nsome code\n```";
        let (html, _) = markdown_to_html(md);

        // Should contain escaped code without highlighting spans
        assert!(html.contains("<pre><code"));
        assert!(html.contains("language-unknown"));
        assert!(html.contains("some code"));
        assert!(!html.contains("class=\"hl-"));
    }

    #[test]
    fn test_inline_code() {
        let md = "Use `cargo run` to start.";
        let (html, _) = markdown_to_html(md);

        assert!(html.contains("<code>cargo run</code>"));
    }

    #[test]
    fn test_image_alt_text() {
        let md = "![Beautiful sunset](sunset.jpg \"Evening sky\")";
        let (html, _) = markdown_to_html(md);

        assert!(html.contains("alt=\"Beautiful sunset\""));
        assert!(html.contains("title=\"Evening sky\""));
        assert!(html.contains("src=\"sunset.jpg\""));
    }

    #[test]
    fn test_image_alt_text_no_title() {
        let md = "![Logo image](logo.png)";
        let (html, _) = markdown_to_html(md);

        assert!(html.contains("alt=\"Logo image\""));
        assert!(html.contains("src=\"logo.png\""));
        assert!(!html.contains("title="));
    }

    #[test]
    fn test_anchor_extraction() {
        let md = r#"# Page Title
## Getting Started
Some intro text.
### Installation
Install steps.
## Configuration
Config details.
#### Deep Heading
"#;
        let (html, anchors) = markdown_to_html(md);

        // h1 should NOT be extracted (page title, not TOC)
        assert!(anchors.iter().all(|a| a.level >= 2));

        // Should have 4 anchors: h2, h3, h2, h4
        assert_eq!(anchors.len(), 4);

        // Check first anchor
        assert_eq!(anchors[0].id, "getting-started");
        assert_eq!(anchors[0].label, "Getting Started");
        assert_eq!(anchors[0].level, 2);

        // Check h3
        assert_eq!(anchors[1].id, "installation");
        assert_eq!(anchors[1].level, 3);

        // Check second h2
        assert_eq!(anchors[2].id, "configuration");
        assert_eq!(anchors[2].level, 2);

        // Check h4
        assert_eq!(anchors[3].id, "deep-heading");
        assert_eq!(anchors[3].level, 4);

        // Verify IDs are in HTML
        assert!(html.contains("id=\"getting-started\""));
        assert!(html.contains("id=\"installation\""));
    }

    #[test]
    fn test_slugify_edge_cases() {
        // Basic case
        assert_eq!(slugify("Hello World"), "hello-world");

        // Multiple spaces → single hyphen
        assert_eq!(slugify("Hello   World"), "hello-world");

        // Special characters → hyphen (apostrophe becomes hyphen)
        assert_eq!(slugify("What's New?"), "what-s-new");

        // Numbers preserved, dot becomes hyphen
        assert_eq!(slugify("Version 2.0"), "version-2-0");

        // Leading/trailing spaces trimmed
        assert_eq!(slugify("  Padded  "), "padded");

        // Mixed case → lowercase
        assert_eq!(slugify("CamelCase"), "camelcase");

        // Consecutive special chars → single hyphen
        assert_eq!(slugify("A -- B"), "a-b");
    }

    #[test]
    fn test_link_url_escaping() {
        // Quote-breaking attack
        let md = r#"[click]("><script>alert(1)</script>)"#;
        let (html, _) = markdown_to_html(md);
        assert!(!html.contains("<script>"), "script tags should be escaped");
        assert!(html.contains("&gt;"), "angle brackets should be escaped");

        // JavaScript URL (should be escaped, not executed)
        let md = r#"[click](javascript:alert(1))"#;
        let (html, _) = markdown_to_html(md);
        assert!(html.contains("href=\"javascript:alert(1)\""));
    }

    #[test]
    fn test_link_title_escaping() {
        let md = r#"[text](url "title with \"quotes\"")"#;
        let (html, _) = markdown_to_html(md);
        assert!(html.contains("&quot;"), "quotes in title should be escaped");
    }

    #[test]
    fn test_image_src_escaping() {
        // Quote-breaking attack in image src
        let md = r#"![alt]("><script>alert(1)</script>)"#;
        let (html, _) = markdown_to_html(md);
        assert!(!html.contains("<script>"), "script tags should be escaped");
        assert!(
            html.contains("&quot;") || html.contains("&gt;"),
            "special chars in src should be escaped"
        );
    }

    #[test]
    fn test_unlabeled_code_block_preserves_quotes() {
        // Code block without language specifier should preserve quotes
        let md = "```\nContent-Security-Policy: default-src 'self';\n```";
        let (html, _) = markdown_to_html(md);

        // Should be inside <pre><code>
        assert!(html.contains("<pre><code>"), "should have code block");
        // Quotes should NOT be escaped (only <, >, & need escaping in code)
        assert!(
            html.contains("'self'"),
            "single quotes should be preserved in code blocks"
        );
        // Should NOT have escaped quotes
        assert!(
            !html.contains("&#39;"),
            "quotes should not be HTML-escaped in code blocks"
        );
    }
}
