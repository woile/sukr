//! Atom feed generation.

use crate::config::SiteConfig;
use crate::content::SiteManifest;
use crate::escape::xml_escape;
use std::path::Path;

/// Generate an Atom 1.0 feed from blog posts in the manifest.
pub fn generate_atom_feed(
    manifest: &SiteManifest,
    config: &SiteConfig,
    content_root: &Path,
) -> String {
    let posts = &manifest.posts;
    let base_url = config.base_url.trim_end_matches('/');

    // Use the most recent post date as feed updated time, or fallback
    let updated = posts
        .first()
        .and_then(|p| p.frontmatter.date.as_ref())
        .map(|d| format!("{}T00:00:00Z", d))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

    let mut entries = String::new();
    for post in posts {
        // Derive URL from output path (e.g., blog/foo.html → /blog/foo.html)
        let relative_path = post.output_path(content_root);
        let post_url = format!("{}/{}", base_url, relative_path.display());
        let post_date = post
            .frontmatter
            .date
            .as_ref()
            .map(|d| format!("{}T00:00:00Z", d))
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        let summary = post
            .frontmatter
            .description
            .as_ref()
            .map(|s| xml_escape(s))
            .unwrap_or_default();

        entries.push_str(&format!(
            r#"  <entry>
    <title>{}</title>
    <link href="{}" rel="alternate"/>
    <id>{}</id>
    <updated>{}</updated>
    <summary>{}</summary>
  </entry>
"#,
            xml_escape(&post.frontmatter.title),
            post_url,
            post_url,
            post_date,
            summary,
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>{}</title>
  <link href="{}" rel="alternate"/>
  <link href="{}/feed.xml" rel="self"/>
  <id>{}/</id>
  <updated>{}</updated>
  <author>
    <name>{}</name>
  </author>
{}
</feed>
"#,
        xml_escape(&config.title),
        base_url,
        base_url,
        base_url,
        updated,
        xml_escape(&config.author),
        entries,
    )
}
