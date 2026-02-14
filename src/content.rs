//! Content discovery and frontmatter parsing.

use crate::error::{Error, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Default weight for content items in navigation and listings.
pub(crate) const DEFAULT_WEIGHT: i64 = 50;

/// High default weight for content that should appear last (e.g., projects).
pub(crate) const DEFAULT_WEIGHT_HIGH: i64 = 99;

/// The type of content being processed.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentKind {
    /// Blog post with full metadata (date, tags, etc.)
    Post,
    /// Standalone page (about, collab)
    Page,
    /// Section index (_index.md)
    Section,
    /// Project card with external link
    Project,
}

/// A navigation menu item discovered from the filesystem.
#[derive(Debug, Clone, Serialize)]
pub struct NavItem {
    /// Display label (from nav_label or title)
    pub label: String,
    /// URL path (e.g., "/blog/index.html" or "/about.html")
    pub path: String,
    /// Sort order (lower = first, default 50)
    pub weight: i64,
    /// Child navigation items (section pages)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NavItem>,
}

/// Parsed frontmatter from a content file.
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_date")]
    pub date: Option<NaiveDate>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Sort order for nav and listings
    #[serde(default)]
    pub weight: Option<i64>,
    /// For project cards: external link
    #[serde(default)]
    pub link_to: Option<String>,
    /// Custom navigation label (defaults to title)
    #[serde(default)]
    pub nav_label: Option<String>,
    /// Section type for template dispatch (e.g., "blog", "projects")
    #[serde(default)]
    pub section_type: Option<String>,
    /// Override template for this content item
    #[serde(default)]
    pub template: Option<String>,
    /// Enable table of contents (anchor nav in sidebar)
    #[serde(default)]
    pub toc: Option<bool>,
    /// Whether this content is a draft (excluded from output)
    #[serde(default)]
    pub draft: bool,
    /// Alternative URL paths that redirect to this content
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// A content item ready for rendering.
#[derive(Debug, Clone)]
pub struct Content {
    pub kind: ContentKind,
    pub frontmatter: Frontmatter,
    pub body: String,
    pub source_path: PathBuf,
    pub slug: String,
}

impl Content {
    /// Load and parse a markdown file with TOML frontmatter.
    pub fn from_path(path: impl AsRef<Path>, kind: ContentKind) -> Result<Self> {
        Self::from_path_inner(path.as_ref(), kind)
    }

    fn from_path_inner(path: &Path, kind: ContentKind) -> Result<Self> {
        let raw = fs::read_to_string(path).map_err(|e| Error::ReadFile {
            path: path.to_path_buf(),
            source: e,
        })?;

        let (toml_block, body) = extract_frontmatter(&raw, path)?;
        let frontmatter = parse_frontmatter(path, &toml_block)?;

        // Derive slug from filename (without extension)
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        Ok(Content {
            kind,
            frontmatter,
            body,
            source_path: path.to_path_buf(),
            slug,
        })
    }

    /// Compute the output path relative to the output directory.
    /// e.g., content/blog/foo.md → blog/foo.html
    pub fn output_path(&self, content_root: &Path) -> PathBuf {
        let relative = self
            .source_path
            .strip_prefix(content_root)
            .unwrap_or(&self.source_path);

        match self.kind {
            ContentKind::Section => {
                // _index.md → parent/index.html (listing pages stay as index.html)
                let parent = relative.parent().unwrap_or(Path::new(""));
                parent.join("index.html")
            },
            _ => {
                // Regular content → parent/slug.html (flat structure)
                let parent = relative.parent().unwrap_or(Path::new(""));
                parent.join(format!("{}.html", self.slug))
            },
        }
    }
}

/// Extract TOML frontmatter block and body from raw content.
/// Frontmatter must be delimited by `+++` at start and end.
fn extract_frontmatter(raw: &str, path: &Path) -> Result<(String, String)> {
    let trimmed = raw.trim_start();

    if !trimmed.starts_with("+++") {
        return Err(Error::Frontmatter {
            path: path.to_path_buf(),
            message: "missing frontmatter delimiter".to_string(),
        });
    }

    // Find the closing +++
    let after_first = &trimmed[3..].trim_start_matches(['\r', '\n']);
    let end_idx = after_first
        .find("\n+++")
        .ok_or_else(|| Error::Frontmatter {
            path: path.to_path_buf(),
            message: "missing closing frontmatter delimiter".to_string(),
        })?;

    let toml_block = after_first[..end_idx].to_string();
    let body = after_first[end_idx + 4..].trim_start().to_string();

    Ok((toml_block, body))
}

/// Parse TOML frontmatter into structured fields.
fn parse_frontmatter(path: &Path, toml_str: &str) -> Result<Frontmatter> {
    toml::from_str(toml_str).map_err(|e| Error::Frontmatter {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

/// Deserialize a TOML native date into `Option<NaiveDate>`.
///
/// TOML native dates (`date = 2026-01-15`) are parsed by the `toml` crate as
/// `toml::value::Datetime`. This deserializer accepts that type, extracts the
/// date component, and constructs a validated `chrono::NaiveDate`.
fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let dt = toml::value::Datetime::deserialize(deserializer)?;
    let date = dt
        .date
        .ok_or_else(|| serde::de::Error::custom("datetime must include a date component"))?;
    NaiveDate::from_ymd_opt(date.year.into(), date.month.into(), date.day.into())
        .ok_or_else(|| serde::de::Error::custom("invalid calendar date"))
        .map(Some)
}

/// Discover navigation items from the content directory structure.
///
/// Rules:
/// - Top-level `.md` files (except `_index.md`) become nav items (pages)
/// - Directories containing `_index.md` become nav items (sections)
/// - Items are sorted by weight (lower first), then alphabetically by label
pub fn discover_nav(content_dir: &Path) -> Result<Vec<NavItem>> {
    let mut nav_items = Vec::new();

    // Read top-level entries in content directory
    let entries = fs::read_dir(content_dir).map_err(|e| Error::ReadFile {
        path: content_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            // Top-level .md file (except _index.md) → page nav item
            if path.extension().is_some_and(|ext| ext == "md") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name != "_index.md" {
                    let content = Content::from_path(&path, ContentKind::Page)?;
                    if !content.frontmatter.draft {
                        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
                        nav_items.push(NavItem {
                            label: content
                                .frontmatter
                                .nav_label
                                .unwrap_or(content.frontmatter.title),
                            path: format!("/{}.html", slug),
                            weight: content.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT),
                            children: Vec::new(),
                        });
                    }
                }
            }
        } else if path.is_dir() {
            // Directory with _index.md → section nav item
            let index_path = path.join("_index.md");
            if index_path.exists() {
                let content = Content::from_path(&index_path, ContentKind::Section)?;
                let dir_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("section");

                // Collect section items as child nav items
                let section = Section {
                    index: content.clone(),
                    name: dir_name.to_string(),
                    section_type: content
                        .frontmatter
                        .section_type
                        .clone()
                        .unwrap_or_else(|| dir_name.to_string()),
                    path: path.clone(),
                };

                let mut children: Vec<NavItem> = section
                    .collect_items()?
                    .into_iter()
                    .map(|item| NavItem {
                        label: item.frontmatter.nav_label.unwrap_or(item.frontmatter.title),
                        path: format!("/{}/{}.html", dir_name, item.slug),
                        weight: item.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT),
                        children: Vec::new(),
                    })
                    .collect();

                // Sort children by weight, then alphabetically
                children
                    .sort_by(|a, b| a.weight.cmp(&b.weight).then_with(|| a.label.cmp(&b.label)));

                nav_items.push(NavItem {
                    label: content
                        .frontmatter
                        .nav_label
                        .unwrap_or(content.frontmatter.title),
                    path: format!("/{}/index.html", dir_name),
                    weight: content.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT),
                    children,
                });
            }
        }
    }

    // Sort by weight, then alphabetically by label
    nav_items.sort_by(|a, b| a.weight.cmp(&b.weight).then_with(|| a.label.cmp(&b.label)));

    Ok(nav_items)
}

/// A discovered section from the content directory.
#[derive(Debug)]
pub struct Section {
    /// The section's index content (_index.md)
    pub index: Content,
    /// Directory name (e.g., "blog", "projects")
    pub name: String,
    /// Section type for template dispatch (from frontmatter or directory name)
    pub section_type: String,
    /// Path to section directory
    pub path: PathBuf,
}

impl Section {
    /// Collect all content items in this section (excluding _index.md and drafts).
    pub fn collect_items(&self) -> Result<Vec<Content>> {
        let mut items = Vec::new();

        for entry in fs::read_dir(&self.path)
            .map_err(|e| Error::ReadFile {
                path: self.path.clone(),
                source: e,
            })?
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|n| n != "_index.md")
            {
                // Determine content kind based on section type
                let kind = match self.section_type.as_str() {
                    "blog" => ContentKind::Post,
                    "projects" => ContentKind::Project,
                    _ => ContentKind::Page,
                };
                let content = Content::from_path(&path, kind)?;
                if !content.frontmatter.draft {
                    items.push(content);
                }
            }
        }

        Ok(items)
    }
}

/// Discover all sections (directories with _index.md) in the content directory.
pub fn discover_sections(content_dir: &Path) -> Result<Vec<Section>> {
    let mut sections = Vec::new();

    let entries = fs::read_dir(content_dir).map_err(|e| Error::ReadFile {
        path: content_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            let index_path = path.join("_index.md");
            if index_path.exists() {
                let index = Content::from_path(&index_path, ContentKind::Section)?;
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("section")
                    .to_string();

                // Section type from frontmatter, or fall back to directory name
                let section_type = index
                    .frontmatter
                    .section_type
                    .clone()
                    .unwrap_or_else(|| name.clone());

                sections.push(Section {
                    index,
                    name,
                    section_type,
                    path,
                });
            }
        }
    }

    // Sort by weight
    sections.sort_by(|a, b| {
        let wa = a.index.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT);
        let wb = b.index.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT);
        wa.cmp(&wb)
    });

    Ok(sections)
}

/// Discover standalone pages (top-level .md files except _index.md and _404.md).
pub fn discover_pages(content_dir: &Path) -> Result<Vec<Content>> {
    let mut pages = Vec::new();

    let entries = fs::read_dir(content_dir).map_err(|e| Error::ReadFile {
        path: content_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "md")
            && path
                .file_name()
                .is_some_and(|n| n != "_index.md" && n != "_404.md")
        {
            let content = Content::from_path(&path, ContentKind::Page)?;
            if !content.frontmatter.draft {
                pages.push(content);
            }
        }
    }

    Ok(pages)
}

/// Complete site content manifest from a single discovery pass.
///
/// Aggregates all content types for use by rendering, feed, and sitemap generation.
#[derive(Debug)]
pub struct SiteManifest {
    /// Homepage content (content/_index.md)
    pub homepage: Content,
    /// Custom 404 page (content/_404.md), if present
    pub page_404: Option<Content>,
    /// All sections (directories with _index.md)
    pub sections: Vec<Section>,
    /// Standalone pages (top-level .md files)
    pub pages: Vec<Content>,
    /// Blog posts for feed generation (items from "blog" sections)
    pub posts: Vec<Content>,
    /// Navigation menu items
    pub nav: Vec<NavItem>,
}

impl SiteManifest {
    /// Discover all site content in a single pass.
    pub fn discover(content_dir: impl AsRef<Path>) -> Result<Self> {
        Self::discover_inner(content_dir.as_ref())
    }

    fn discover_inner(content_dir: &Path) -> Result<Self> {
        // Load homepage
        let homepage_path = content_dir.join("_index.md");
        let homepage = Content::from_path(&homepage_path, ContentKind::Section)?;

        // Load 404 page if present
        let page_404_path = content_dir.join("_404.md");
        let page_404 = if page_404_path.exists() {
            Some(Content::from_path(&page_404_path, ContentKind::Page)?)
        } else {
            None
        };

        // Discover navigation
        let nav = discover_nav(content_dir)?;

        // Discover sections
        let sections = discover_sections(content_dir)?;

        // Collect section items and identify blog posts
        let mut posts = Vec::new();
        for section in &sections {
            if section.section_type == "blog" {
                let mut items = section.collect_items()?;
                // Sort blog posts by date, newest first
                items.sort_by(|a, b| b.frontmatter.date.cmp(&a.frontmatter.date));
                posts.extend(items);
            }
        }

        // Discover standalone pages
        let pages = discover_pages(content_dir)?;

        Ok(SiteManifest {
            homepage,
            page_404,
            sections,
            pages,
            posts,
            nav,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    fn write_frontmatter(path: &Path, title: &str, weight: Option<i64>, nav_label: Option<&str>) {
        let mut content = format!("+++\ntitle = \"{}\"\n", title);
        if let Some(w) = weight {
            content.push_str(&format!("weight = {}\n", w));
        }
        if let Some(label) = nav_label {
            content.push_str(&format!("nav_label = \"{}\"\n", label));
        }
        content.push_str("+++\n\nBody content.");
        fs::write(path, content).expect("failed to write test file");
    }

    #[test]
    fn test_discover_nav_finds_pages() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        // Create top-level page
        write_frontmatter(&content_dir.join("about.md"), "About Me", None, None);

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].label, "About Me");
        assert_eq!(nav[0].path, "/about.html");
    }

    #[test]
    fn test_discover_nav_finds_sections() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        // Create section directory with _index.md
        let blog_dir = content_dir.join("blog");
        fs::create_dir(&blog_dir).expect("failed to create blog dir");
        write_frontmatter(&blog_dir.join("_index.md"), "Blog", None, None);

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].label, "Blog");
        assert_eq!(nav[0].path, "/blog/index.html");
    }

    #[test]
    fn test_discover_nav_excludes_root_index() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        // Create _index.md at root (should be excluded from nav)
        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        write_frontmatter(&content_dir.join("about.md"), "About", None, None);

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].label, "About");
    }

    #[test]
    fn test_discover_nav_sorts_by_weight() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("about.md"), "About", Some(30), None);
        write_frontmatter(&content_dir.join("contact.md"), "Contact", Some(10), None);
        write_frontmatter(&content_dir.join("blog.md"), "Blog", Some(20), None);

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 3);
        assert_eq!(nav[0].label, "Contact"); // weight 10
        assert_eq!(nav[1].label, "Blog"); // weight 20
        assert_eq!(nav[2].label, "About"); // weight 30
    }

    #[test]
    fn test_discover_nav_uses_nav_label() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(
            &content_dir.join("about.md"),
            "About The Author",
            None,
            Some("About"),
        );

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].label, "About"); // Uses nav_label, not title
    }

    #[test]
    fn test_discover_nav_populates_section_children() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        // Create section directory with _index.md and child pages
        let features_dir = content_dir.join("features");
        fs::create_dir(&features_dir).expect("failed to create features dir");
        write_frontmatter(&features_dir.join("_index.md"), "Features", Some(10), None);
        write_frontmatter(
            &features_dir.join("templates.md"),
            "Templates",
            Some(20),
            None,
        );
        write_frontmatter(
            &features_dir.join("highlights.md"),
            "Syntax Highlighting",
            Some(10),
            None,
        );

        let nav = discover_nav(content_dir).expect("discover_nav failed");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].label, "Features");
        assert_eq!(nav[0].children.len(), 2);

        // Children should be sorted by weight
        assert_eq!(nav[0].children[0].label, "Syntax Highlighting"); // weight 10
        assert_eq!(nav[0].children[0].path, "/features/highlights.html");
        assert_eq!(nav[0].children[1].label, "Templates"); // weight 20
        assert_eq!(nav[0].children[1].path, "/features/templates.html");
    }
    // =========================================================================
    // discover_sections tests
    // =========================================================================

    fn write_section_index(
        path: &Path,
        title: &str,
        section_type: Option<&str>,
        weight: Option<i64>,
    ) {
        let mut content = format!("+++\ntitle = \"{}\"\n", title);
        if let Some(st) = section_type {
            content.push_str(&format!("section_type = \"{}\"\n", st));
        }
        if let Some(w) = weight {
            content.push_str(&format!("weight = {}\n", w));
        }
        content.push_str("+++\nSection content.\n");
        fs::write(path, content).expect("failed to write section index");
    }

    #[test]
    fn test_discover_sections_finds_directories() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        // Create two sections
        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(&content_dir.join("blog/_index.md"), "Blog", None, None);

        fs::create_dir(content_dir.join("projects")).unwrap();
        write_section_index(
            &content_dir.join("projects/_index.md"),
            "Projects",
            None,
            None,
        );

        let sections = discover_sections(content_dir).expect("discover_sections failed");
        assert_eq!(sections.len(), 2);

        let names: Vec<_> = sections.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"blog"));
        assert!(names.contains(&"projects"));
    }

    #[test]
    fn test_discover_sections_uses_section_type_from_frontmatter() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        fs::create_dir(content_dir.join("writings")).unwrap();
        write_section_index(
            &content_dir.join("writings/_index.md"),
            "My Writings",
            Some("blog"),
            None,
        );

        let sections = discover_sections(content_dir).expect("discover_sections failed");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "writings");
        assert_eq!(sections[0].section_type, "blog"); // From frontmatter, not dir name
    }

    #[test]
    fn test_discover_sections_falls_back_to_dir_name() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        fs::create_dir(content_dir.join("gallery")).unwrap();
        write_section_index(
            &content_dir.join("gallery/_index.md"),
            "Gallery",
            None,
            None,
        );

        let sections = discover_sections(content_dir).expect("discover_sections failed");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].section_type, "gallery"); // Falls back to dir name
    }

    #[test]
    fn test_discover_sections_sorts_by_weight() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(&content_dir.join("blog/_index.md"), "Blog", None, Some(20));

        fs::create_dir(content_dir.join("projects")).unwrap();
        write_section_index(
            &content_dir.join("projects/_index.md"),
            "Projects",
            None,
            Some(10),
        );

        let sections = discover_sections(content_dir).expect("discover_sections failed");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "projects"); // weight 10
        assert_eq!(sections[1].name, "blog"); // weight 20
    }

    #[test]
    fn test_section_collect_items() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(
            &content_dir.join("blog/_index.md"),
            "Blog",
            Some("blog"),
            None,
        );
        write_frontmatter(&content_dir.join("blog/post1.md"), "Post 1", None, None);
        write_frontmatter(&content_dir.join("blog/post2.md"), "Post 2", None, None);

        let sections = discover_sections(content_dir).expect("discover_sections failed");
        assert_eq!(sections.len(), 1);

        let items = sections[0].collect_items().expect("collect_items failed");
        assert_eq!(items.len(), 2);

        let titles: Vec<_> = items.iter().map(|c| c.frontmatter.title.as_str()).collect();
        assert!(titles.contains(&"Post 1"));
        assert!(titles.contains(&"Post 2"));
    }

    // =========================================================================
    // SiteManifest tests
    // =========================================================================

    #[test]
    fn test_manifest_discovers_homepage() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);

        let manifest = SiteManifest::discover(content_dir).expect("discover failed");
        assert_eq!(manifest.homepage.frontmatter.title, "Home");
    }

    #[test]
    fn test_manifest_discovers_sections() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(
            &content_dir.join("blog/_index.md"),
            "Blog",
            Some("blog"),
            None,
        );

        let manifest = SiteManifest::discover(content_dir).expect("discover failed");
        assert_eq!(manifest.sections.len(), 1);
        assert_eq!(manifest.sections[0].name, "blog");
    }

    #[test]
    fn test_manifest_discovers_pages() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        write_frontmatter(&content_dir.join("about.md"), "About", None, None);
        write_frontmatter(&content_dir.join("contact.md"), "Contact", None, None);

        let manifest = SiteManifest::discover(content_dir).expect("discover failed");
        assert_eq!(manifest.pages.len(), 2);

        let titles: Vec<_> = manifest
            .pages
            .iter()
            .map(|c| c.frontmatter.title.as_str())
            .collect();
        assert!(titles.contains(&"About"));
        assert!(titles.contains(&"Contact"));
    }

    #[test]
    fn test_manifest_collects_blog_posts() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(
            &content_dir.join("blog/_index.md"),
            "Blog",
            Some("blog"),
            None,
        );

        // Create blog posts with dates
        let post1 = "+++\ntitle = \"Post 1\"\ndate = 2026-01-15\n+++\nContent.".to_string();
        let post2 = "+++\ntitle = \"Post 2\"\ndate = 2026-01-20\n+++\nContent.".to_string();
        fs::write(content_dir.join("blog/post1.md"), &post1).unwrap();
        fs::write(content_dir.join("blog/post2.md"), &post2).unwrap();

        let manifest = SiteManifest::discover(content_dir).expect("discover failed");
        assert_eq!(manifest.posts.len(), 2);

        // Should be sorted by date, newest first
        assert_eq!(manifest.posts[0].frontmatter.title, "Post 2"); // 2026-01-20
        assert_eq!(manifest.posts[1].frontmatter.title, "Post 1"); // 2026-01-15
    }

    #[test]
    fn test_manifest_discovers_nav() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        write_frontmatter(&content_dir.join("about.md"), "About", Some(10), None);
        fs::create_dir(content_dir.join("blog")).unwrap();
        write_section_index(&content_dir.join("blog/_index.md"), "Blog", None, Some(20));

        let manifest = SiteManifest::discover(content_dir).expect("discover failed");
        assert_eq!(manifest.nav.len(), 2);

        // Nav should be sorted by weight
        assert_eq!(manifest.nav[0].label, "About"); // weight 10
        assert_eq!(manifest.nav[1].label, "Blog"); // weight 20
    }

    fn write_draft(path: &Path, title: &str) {
        let content = format!(
            "+++\ntitle = \"{}\"\ndraft = true\n+++\n\nDraft content.",
            title
        );
        fs::write(path, content).expect("failed to write draft file");
    }

    #[test]
    fn test_collect_items_excludes_drafts() {
        let dir = create_test_dir();
        let section_dir = dir.path().join("features");
        fs::create_dir(&section_dir).unwrap();
        write_frontmatter(&section_dir.join("_index.md"), "Features", None, None);
        write_frontmatter(&section_dir.join("visible.md"), "Visible", None, None);
        write_draft(&section_dir.join("hidden.md"), "Hidden");

        let section = Section {
            index: Content::from_path(&section_dir.join("_index.md"), ContentKind::Section)
                .unwrap(),
            name: "features".to_string(),
            section_type: "features".to_string(),
            path: section_dir,
        };

        let items = section.collect_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].frontmatter.title, "Visible");
    }

    #[test]
    fn test_discover_nav_excludes_drafts() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("about.md"), "About", Some(10), None);
        write_draft(&content_dir.join("secret.md"), "Secret Page");

        let nav = discover_nav(content_dir).unwrap();
        assert_eq!(nav.len(), 1, "draft page should not appear in nav");
        assert_eq!(nav[0].label, "About");
    }

    #[test]
    fn test_discover_pages_excludes_drafts() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("about.md"), "About", None, None);
        write_draft(&content_dir.join("wip.md"), "Work in Progress");

        let pages = discover_pages(content_dir).unwrap();
        assert_eq!(pages.len(), 1, "draft page should not appear in pages");
        assert_eq!(pages[0].frontmatter.title, "About");
    }

    #[test]
    fn test_discover_pages_excludes_404() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("about.md"), "About", None, None);
        fs::write(
            content_dir.join("_404.md"),
            "+++\ntitle = \"Not Found\"\n+++\n\n404 body.",
        )
        .unwrap();

        let pages = discover_pages(content_dir).unwrap();
        assert_eq!(pages.len(), 1, "_404.md should not appear in pages");
        assert_eq!(pages[0].frontmatter.title, "About");
    }

    #[test]
    fn test_manifest_detects_404() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);
        fs::write(
            content_dir.join("_404.md"),
            "+++\ntitle = \"Page Not Found\"\n+++\n\n404 body.",
        )
        .unwrap();

        let manifest = SiteManifest::discover(content_dir).unwrap();
        assert!(manifest.page_404.is_some(), "page_404 should be populated");
        assert_eq!(
            manifest.page_404.unwrap().frontmatter.title,
            "Page Not Found"
        );
    }

    #[test]
    fn test_manifest_without_404() {
        let dir = create_test_dir();
        let content_dir = dir.path();

        write_frontmatter(&content_dir.join("_index.md"), "Home", None, None);

        let manifest = SiteManifest::discover(content_dir).unwrap();
        assert!(
            manifest.page_404.is_none(),
            "page_404 should be None when _404.md absent"
        );
    }
}
