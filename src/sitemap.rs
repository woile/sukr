//! XML sitemap generation for SEO.

use crate::config::SiteConfig;
use crate::content::SiteManifest;
use crate::escape::xml_escape;
use std::path::Path;

/// A URL entry for the sitemap.
pub(crate) struct SitemapEntry {
    /// Absolute URL (e.g., "https://example.com/blog/post.html")
    pub loc: String,
    /// Optional last modification date in W3C format (YYYY-MM-DD)
    pub lastmod: Option<String>,
}

/// Generate an XML sitemap from the site manifest.
///
/// Includes:
/// - Homepage
/// - Section indices
/// - Section items (posts, projects, etc.)
/// - Standalone pages
pub fn generate_sitemap(
    manifest: &SiteManifest,
    config: &SiteConfig,
    content_root: &Path,
    tag_names: &[String],
) -> String {
    let base_url = config.base_url.trim_end_matches('/');
    let mut entries = Vec::new();

    // Homepage
    entries.push(SitemapEntry {
        loc: format!("{}/index.html", base_url),
        lastmod: None,
    });

    // Sections and their items
    for section in &manifest.sections {
        // Section index
        entries.push(SitemapEntry {
            loc: format!("{}/{}/index.html", base_url, section.name),
            lastmod: section.index.frontmatter.date.map(|d| d.to_string()),
        });

        // Section items
        for item in &section.items {
            let relative_path = &item.output_path;
            entries.push(SitemapEntry {
                loc: format!("{}/{}", base_url, relative_path.display()),
                lastmod: item.frontmatter.date.map(|d| d.to_string()),
            });
        }
    }

    // Standalone pages
    for page in &manifest.pages {
        let relative_path = &page.output_path;
        entries.push(SitemapEntry {
            loc: format!("{}/{}", base_url, relative_path.display()),
            lastmod: page.frontmatter.date.map(|d| d.to_string()),
        });
    }

    // Tag listing pages
    for tag in tag_names {
        entries.push(SitemapEntry {
            loc: format!("{}/tags/{}.html", base_url, tag),
            lastmod: None,
        });
    }

    build_sitemap_xml(&entries)
}

/// Build the XML sitemap string from entries.
fn build_sitemap_xml(entries: &[SitemapEntry]) -> String {
    let mut urls = String::new();

    for entry in entries {
        urls.push_str("  <url>\n");
        urls.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&entry.loc)));
        if let Some(ref date) = entry.lastmod {
            urls.push_str(&format!("    <lastmod>{}</lastmod>\n", xml_escape(date)));
        }
        urls.push_str("  </url>\n");
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{}
</urlset>
"#,
        urls.trim_end()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sitemap_xml_single_entry() {
        let entries = vec![SitemapEntry {
            loc: "https://example.com/index.html".to_string(),
            lastmod: None,
        }];

        let xml = build_sitemap_xml(&entries);

        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#));
        assert!(xml.contains("<urlset xmlns="));
        assert!(xml.contains("<url>"));
        assert!(xml.contains("<loc>https://example.com/index.html</loc>"));
        assert!(xml.contains("</urlset>"));
        assert!(!xml.contains("<lastmod>")); // No lastmod when None
    }

    #[test]
    fn test_build_sitemap_xml_with_lastmod() {
        let entries = vec![SitemapEntry {
            loc: "https://example.com/blog/post.html".to_string(),
            lastmod: Some("2026-01-31".to_string()),
        }];

        let xml = build_sitemap_xml(&entries);

        assert!(xml.contains("<loc>https://example.com/blog/post.html</loc>"));
        assert!(xml.contains("<lastmod>2026-01-31</lastmod>"));
    }

    #[test]
    fn test_build_sitemap_xml_multiple_entries() {
        let entries = vec![
            SitemapEntry {
                loc: "https://example.com/index.html".to_string(),
                lastmod: None,
            },
            SitemapEntry {
                loc: "https://example.com/about.html".to_string(),
                lastmod: Some("2026-01-15".to_string()),
            },
            SitemapEntry {
                loc: "https://example.com/blog/index.html".to_string(),
                lastmod: None,
            },
        ];

        let xml = build_sitemap_xml(&entries);

        // Count url elements
        let url_count = xml.matches("<url>").count();
        assert_eq!(url_count, 3);

        // Verify all URLs present
        assert!(xml.contains("https://example.com/index.html"));
        assert!(xml.contains("https://example.com/about.html"));
        assert!(xml.contains("https://example.com/blog/index.html"));
    }

    #[test]
    fn test_build_sitemap_xml_escapes_special_chars() {
        let entries = vec![SitemapEntry {
            loc: "https://example.com/search?q=foo&bar=baz".to_string(),
            lastmod: None,
        }];

        let xml = build_sitemap_xml(&entries);

        // & should be escaped
        assert!(xml.contains("&amp;"));
        assert!(!xml.contains("?q=foo&bar")); // Raw & should not appear
    }

    #[test]
    fn test_sitemap_includes_tag_pages() {
        let tag_names = vec!["rust".to_string(), "web".to_string()];
        let entries: Vec<SitemapEntry> = tag_names
            .iter()
            .map(|tag| SitemapEntry {
                loc: format!("https://example.com/tags/{}.html", tag),
                lastmod: None,
            })
            .collect();

        let xml = build_sitemap_xml(&entries);

        assert!(xml.contains("https://example.com/tags/rust.html"));
        assert!(xml.contains("https://example.com/tags/web.html"));
        assert_eq!(xml.matches("<url>").count(), 2);
    }
}
