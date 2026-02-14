//! Tera-based template engine for runtime HTML generation.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use tera::{Context, Tera, Value};

use crate::config::SiteConfig;
use crate::content::{Content, NavItem};
use crate::error::{Error, Result};
use crate::render::Anchor;

/// Runtime template engine wrapping Tera.
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Load templates from a directory (glob pattern: `templates/**/*`).
    pub fn new(template_dir: &Path) -> Result<Self> {
        let pattern = template_dir.join("**/*").display().to_string();
        let mut tera = Tera::new(&pattern).map_err(Error::TemplateLoad)?;

        // Register custom filters
        tera.register_filter("strip_parens", strip_parens_filter);

        Ok(Self { tera })
    }

    /// Render a template by name with the given context.
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String> {
        self.tera
            .render(template_name, context)
            .map_err(|e| Error::TemplateRender {
                template: template_name.to_string(),
                source: e,
            })
    }

    /// Render a standalone page (about, collab, etc.).
    pub fn render_page(
        &self,
        content: &Content,
        html_body: &str,
        page_path: &str,
        config: &SiteConfig,
        nav: &[NavItem],
        anchors: &[Anchor],
    ) -> Result<String> {
        let mut ctx = self.base_context(page_path, config, nav);
        ctx.insert("title", &content.frontmatter.title);
        ctx.insert(
            "page",
            &FrontmatterContext::new(&content.frontmatter, config),
        );
        ctx.insert("content", html_body);
        ctx.insert("anchors", anchors);
        self.render("page.html", &ctx)
    }

    /// Render a content item (blog post, project, etc.).
    pub fn render_content(
        &self,
        content: &Content,
        html_body: &str,
        page_path: &str,
        config: &SiteConfig,
        nav: &[NavItem],
        anchors: &[Anchor],
    ) -> Result<String> {
        let template = content
            .frontmatter
            .template
            .as_deref()
            .unwrap_or("content/default.html");
        let mut ctx = self.base_context(page_path, config, nav);
        ctx.insert("title", &content.frontmatter.title);
        ctx.insert(
            "page",
            &FrontmatterContext::new(&content.frontmatter, config),
        );
        ctx.insert("content", html_body);
        ctx.insert("anchors", anchors);
        self.render(template, &ctx)
    }

    /// Render a section index page (blog index, projects index).
    pub fn render_section(
        &self,
        section: &Content,
        section_type: &str,
        items: &[ContentContext],
        page_path: &str,
        config: &SiteConfig,
        nav: &[NavItem],
    ) -> Result<String> {
        let template = format!("section/{}.html", section_type);

        let mut ctx = self.base_context(page_path, config, nav);
        ctx.insert("title", &section.frontmatter.title);
        ctx.insert(
            "section",
            &FrontmatterContext::new(&section.frontmatter, config),
        );
        ctx.insert("items", items);
        self.render(&template, &ctx)
    }

    /// Build base context with common variables.
    fn base_context(&self, page_path: &str, config: &SiteConfig, nav: &[NavItem]) -> Context {
        let mut ctx = Context::new();
        ctx.insert("config", &ConfigContext::from(config));
        ctx.insert("nav", nav);
        ctx.insert("page_path", page_path);
        ctx.insert("prefix", &relative_prefix(page_path));
        // Trimmed base_url for canonical links
        ctx.insert("base_url", config.base_url.trim_end_matches('/'));
        ctx
    }
}

/// Compute relative path prefix based on page depth.
fn relative_prefix(page_path: &str) -> String {
    let depth = page_path.matches('/').count().saturating_sub(1);
    if depth == 0 {
        ".".to_string()
    } else {
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    }
}

// ============================================================================
// Context structs for Tera serialization
// ============================================================================

/// Site config context for templates.
#[derive(Serialize)]
pub struct ConfigContext {
    pub title: String,
    pub author: String,
    pub base_url: String,
    /// Whether to display nested navigation
    pub nested_nav: bool,
}

impl From<&SiteConfig> for ConfigContext {
    fn from(config: &SiteConfig) -> Self {
        Self {
            title: config.title.clone(),
            author: config.author.clone(),
            base_url: config.base_url.clone(),
            nested_nav: config.nav.nested,
        }
    }
}

/// Frontmatter context for templates.
#[derive(Serialize)]
pub struct FrontmatterContext {
    pub title: String,
    pub description: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub weight: Option<i64>,
    pub link_to: Option<String>,
    /// Enable table of contents (anchor nav in sidebar)
    pub toc: bool,
    /// Whether this content is a draft
    pub draft: bool,
    /// Alternative URL paths
    pub aliases: Vec<String>,
}

impl FrontmatterContext {
    /// Create context from frontmatter with config fallback for toc.
    pub fn new(fm: &crate::content::Frontmatter, config: &SiteConfig) -> Self {
        Self {
            title: fm.title.clone(),
            description: fm.description.clone(),
            date: fm.date.map(|d| d.to_string()),
            tags: fm.tags.clone(),
            weight: fm.weight,
            link_to: fm.link_to.clone(),
            toc: fm.toc.unwrap_or(config.nav.toc),
            draft: fm.draft,
            aliases: fm.aliases.clone(),
        }
    }
}

/// Content item context for section listings.
#[derive(Serialize)]
pub struct ContentContext {
    pub frontmatter: FrontmatterContext,
    pub body: String,
    pub slug: String,
    pub path: String,
}

impl ContentContext {
    pub fn from_content(content: &Content, content_dir: &Path, config: &SiteConfig) -> Self {
        Self {
            frontmatter: FrontmatterContext::new(&content.frontmatter, config),
            body: content.body.clone(),
            slug: content.slug.clone(),
            path: format!("/{}", content.output_path(content_dir).display()),
        }
    }
}

/// Tera filter to strip parenthetical text from strings.
/// E.g., "Content (in Section)" → "Content"
fn strip_parens_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    match value {
        Value::String(s) => {
            // Find opening paren and trim everything from there
            let result = if let Some(pos) = s.find('(') {
                s[..pos].trim().to_string()
            } else {
                s.clone()
            };
            Ok(Value::String(result))
        },
        _ => Ok(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_prefix_root() {
        assert_eq!(relative_prefix("/index.html"), ".");
    }

    #[test]
    fn test_relative_prefix_depth_1() {
        assert_eq!(relative_prefix("/blog/index.html"), "..");
    }

    #[test]
    fn test_relative_prefix_depth_2() {
        assert_eq!(relative_prefix("/blog/posts/foo.html"), "../..");
    }

    #[test]
    fn test_toc_config_fallback() {
        use crate::content::Frontmatter;

        // Create configs with different toc defaults
        let config_toc_true = SiteConfig {
            title: "Test".to_string(),
            author: "Test".to_string(),
            base_url: "https://test.com".to_string(),
            paths: crate::config::PathsConfig::default(),
            nav: crate::config::NavConfig {
                nested: false,
                toc: true,
            },
        };

        let config_toc_false = SiteConfig {
            title: "Test".to_string(),
            author: "Test".to_string(),
            base_url: "https://test.com".to_string(),
            paths: crate::config::PathsConfig::default(),
            nav: crate::config::NavConfig {
                nested: false,
                toc: false,
            },
        };

        // Frontmatter with explicit toc: true
        let fm_explicit_true = Frontmatter {
            title: "Test".to_string(),
            description: None,
            date: None,
            tags: vec![],
            weight: None,
            link_to: None,
            nav_label: None,
            section_type: None,
            template: None,
            toc: Some(true),
            draft: false,
            aliases: vec![],
        };

        // Frontmatter with explicit toc: false
        let fm_explicit_false = Frontmatter {
            title: "Test".to_string(),
            description: None,
            date: None,
            tags: vec![],
            weight: None,
            link_to: None,
            nav_label: None,
            section_type: None,
            template: None,
            toc: Some(false),
            draft: false,
            aliases: vec![],
        };

        // Frontmatter with no toc specified (None)
        let fm_none = Frontmatter {
            title: "Test".to_string(),
            description: None,
            date: None,
            tags: vec![],
            weight: None,
            link_to: None,
            nav_label: None,
            section_type: None,
            template: None,
            toc: None,
            draft: false,
            aliases: vec![],
        };

        // Explicit true overrides config false
        let ctx = FrontmatterContext::new(&fm_explicit_true, &config_toc_false);
        assert!(
            ctx.toc,
            "explicit toc: true should override config toc: false"
        );

        // Explicit false overrides config true
        let ctx = FrontmatterContext::new(&fm_explicit_false, &config_toc_true);
        assert!(
            !ctx.toc,
            "explicit toc: false should override config toc: true"
        );

        // None falls back to config true
        let ctx = FrontmatterContext::new(&fm_none, &config_toc_true);
        assert!(ctx.toc, "toc: None should fall back to config toc: true");

        // None falls back to config false
        let ctx = FrontmatterContext::new(&fm_none, &config_toc_false);
        assert!(!ctx.toc, "toc: None should fall back to config toc: false");
    }
}
