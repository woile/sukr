//! sukr - Minimal static site compiler.
//!
//! Suckless, Rust, zero JS. Transforms markdown into static HTML.

mod config;
mod content;
mod css;
mod error;
mod escape;
mod feed;
mod highlight;
mod math;
mod mermaid;
mod render;
mod sitemap;
mod template_engine;

use crate::content::{Content, ContentKind, DEFAULT_WEIGHT, DEFAULT_WEIGHT_HIGH, NavItem};
use crate::error::{Error, Result};
use crate::template_engine::{ContentContext, TemplateEngine};
use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
sukr - Minimal static site compiler

USAGE:
    sukr [OPTIONS]

OPTIONS:
    -c, --config <FILE>  Path to site.toml config file (default: ./site.toml)
    -h, --help           Print this help message
";

fn main() {
    match parse_args() {
        Ok(Some(config_path)) => {
            if let Err(e) = run(&config_path) {
                eprintln!("error: {e}");
                // Print full error chain
                let mut source = std::error::Error::source(&e);
                while let Some(cause) = source {
                    eprintln!("  caused by: {cause}");
                    source = std::error::Error::source(cause);
                }
                std::process::exit(1);
            }
        },
        Ok(None) => {}, // --help was printed
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        },
    }
}

/// Parse command-line arguments. Returns None if --help was requested.
fn parse_args() -> std::result::Result<Option<PathBuf>, String> {
    let args: Vec<_> = std::env::args().collect();
    let mut config_path = PathBuf::from("site.toml");
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(None);
            },
            "-c" | "--config" => {
                if i + 1 >= args.len() {
                    return Err("--config requires an argument".to_string());
                }
                config_path = PathBuf::from(&args[i + 1]);
                i += 2;
            },
            arg => {
                return Err(format!("unknown argument: {arg}"));
            },
        }
    }

    Ok(Some(config_path))
}

fn run(config_path: &Path) -> Result<()> {
    // Load site configuration
    let config = config::SiteConfig::load(config_path)?;

    // Resolve paths relative to config file location
    let base_dir = config_path.parent().unwrap_or(Path::new("."));
    let content_dir = base_dir.join(&config.paths.content);
    let output_dir = base_dir.join(&config.paths.output);
    let static_dir = base_dir.join(&config.paths.static_dir);
    let template_dir = base_dir.join(&config.paths.templates);

    if !content_dir.exists() {
        return Err(Error::ContentDirNotFound(content_dir.to_path_buf()));
    }

    // Load Tera templates
    let engine = TemplateEngine::new(&template_dir)?;

    // Discover all site content in a single pass
    let manifest = content::SiteManifest::discover(&content_dir)?;

    // 0. Copy static assets
    copy_static_assets(&static_dir, &output_dir)?;

    // 1. Process all sections
    for section in &manifest.sections {
        eprintln!("processing section: {}", section.name);

        // Collect and sort items in this section
        let mut items = section.collect_items()?;

        // Sort based on section type
        match section.section_type.as_str() {
            "blog" => {
                // Blog: sort by date, newest first
                items.sort_by(|a, b| b.frontmatter.date.cmp(&a.frontmatter.date));
            },
            "projects" => {
                // Projects: sort by weight
                items.sort_by(|a, b| {
                    a.frontmatter
                        .weight
                        .unwrap_or(DEFAULT_WEIGHT_HIGH)
                        .cmp(&b.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT_HIGH))
                });
            },
            _ => {
                // Default: sort by weight then title
                items.sort_by(|a, b| {
                    a.frontmatter
                        .weight
                        .unwrap_or(DEFAULT_WEIGHT)
                        .cmp(&b.frontmatter.weight.unwrap_or(DEFAULT_WEIGHT))
                        .then_with(|| a.frontmatter.title.cmp(&b.frontmatter.title))
                });
            },
        }

        // Render individual content pages for all sections
        for item in &items {
            eprintln!("  processing: {}", item.slug);
            let (html_body, anchors) = render::markdown_to_html(&item.body);
            let page_path = format!("/{}", item.output_path(&content_dir).display());
            let html = engine.render_content(
                item,
                &html_body,
                &page_path,
                &config,
                &manifest.nav,
                &anchors,
            )?;
            write_output(&output_dir, &content_dir, item, html)?;
        }

        // Render section index
        let page_path = format!("/{}/index.html", section.name);
        let item_contexts: Vec<_> = items
            .iter()
            .map(|c| ContentContext::from_content(c, &content_dir, &config))
            .collect();
        let html = engine.render_section(
            &section.index,
            &section.section_type,
            &item_contexts,
            &page_path,
            &config,
            &manifest.nav,
        );

        let out_path = output_dir.join(&section.name).join("index.html");
        fs::create_dir_all(out_path.parent().unwrap()).map_err(|e| Error::CreateDir {
            path: out_path.parent().unwrap().to_path_buf(),
            source: e,
        })?;
        fs::write(&out_path, html?).map_err(|e| Error::WriteFile {
            path: out_path.clone(),
            source: e,
        })?;
        eprintln!("  → {}", out_path.display());
    }

    // 2. Generate Atom feed (blog posts only)
    if !manifest.posts.is_empty() {
        generate_feed(&output_dir, &manifest, &config, &content_dir)?;
    }

    // 3. Process standalone pages
    process_pages(&content_dir, &output_dir, &config, &manifest.nav, &engine)?;

    // 4. Generate homepage
    generate_homepage(&manifest, &output_dir, &config, &engine)?;

    // 5. Generate sitemap
    generate_sitemap_file(&output_dir, &manifest, &config, &content_dir)?;

    eprintln!("done!");
    Ok(())
}

/// Generate the Atom feed
fn generate_feed(
    output_dir: &Path,
    manifest: &content::SiteManifest,
    config: &config::SiteConfig,
    content_dir: &Path,
) -> Result<()> {
    let out_path = output_dir.join("feed.xml");
    eprintln!("generating: {}", out_path.display());

    let feed_xml = feed::generate_atom_feed(manifest, config, content_dir);

    fs::write(&out_path, feed_xml).map_err(|e| Error::WriteFile {
        path: out_path.clone(),
        source: e,
    })?;

    eprintln!("  → {}", out_path.display());
    Ok(())
}

/// Generate the XML sitemap
fn generate_sitemap_file(
    output_dir: &Path,
    manifest: &content::SiteManifest,
    config: &config::SiteConfig,
    content_dir: &Path,
) -> Result<()> {
    let out_path = output_dir.join("sitemap.xml");
    eprintln!("generating: {}", out_path.display());

    let sitemap_xml = sitemap::generate_sitemap(manifest, config, content_dir);

    fs::write(&out_path, sitemap_xml).map_err(|e| Error::WriteFile {
        path: out_path.clone(),
        source: e,
    })?;

    eprintln!("  → {}", out_path.display());
    Ok(())
}

/// Process standalone pages in content/ (top-level .md files excluding _index.md)
fn process_pages(
    content_dir: &Path,
    output_dir: &Path,
    config: &config::SiteConfig,
    nav: &[NavItem],
    engine: &TemplateEngine,
) -> Result<()> {
    // Dynamically discover top-level .md files (except _index.md)
    let entries = fs::read_dir(content_dir).map_err(|e| Error::ReadFile {
        path: content_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "md")
            && path.file_name().is_some_and(|n| n != "_index.md")
        {
            eprintln!("processing: {}", path.display());

            let content = Content::from_path(&path, ContentKind::Page)?;
            let (html_body, anchors) = render::markdown_to_html(&content.body);
            let page_path = format!("/{}", content.output_path(content_dir).display());
            let html =
                engine.render_page(&content, &html_body, &page_path, config, nav, &anchors)?;

            write_output(output_dir, content_dir, &content, html)?;
        }
    }
    Ok(())
}

/// Generate the homepage from manifest.homepage
fn generate_homepage(
    manifest: &content::SiteManifest,
    output_dir: &Path,
    config: &config::SiteConfig,
    engine: &TemplateEngine,
) -> Result<()> {
    eprintln!("generating: homepage");

    let (html_body, anchors) = render::markdown_to_html(&manifest.homepage.body);
    let html = engine.render_page(
        &manifest.homepage,
        &html_body,
        "/index.html",
        config,
        &manifest.nav,
        &anchors,
    )?;

    let out_path = output_dir.join("index.html");

    fs::create_dir_all(output_dir).map_err(|e| Error::CreateDir {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    fs::write(&out_path, html).map_err(|e| Error::WriteFile {
        path: out_path.clone(),
        source: e,
    })?;

    eprintln!("  → {}", out_path.display());
    Ok(())
}

/// Write a content item to its output path
fn write_output(
    output_dir: &Path,
    content_dir: &Path,
    content: &Content,
    html: String,
) -> Result<()> {
    let out_path = output_dir.join(content.output_path(content_dir));
    let out_dir = out_path.parent().unwrap();

    fs::create_dir_all(out_dir).map_err(|e| Error::CreateDir {
        path: out_dir.to_path_buf(),
        source: e,
    })?;

    fs::write(&out_path, html).map_err(|e| Error::WriteFile {
        path: out_path.clone(),
        source: e,
    })?;

    eprintln!("  → {}", out_path.display());
    Ok(())
}

/// Copy static assets (CSS, images, etc.) to output directory.
/// CSS files are minified before writing.
fn copy_static_assets(static_dir: &Path, output_dir: &Path) -> Result<()> {
    use crate::css::bundle_css;

    if !static_dir.exists() {
        return Ok(()); // No static dir is fine
    }

    fs::create_dir_all(output_dir).map_err(|e| Error::CreateDir {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    for entry in walkdir::WalkDir::new(static_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let src = entry.path();
        let relative = src.strip_prefix(static_dir).unwrap();
        let dest = output_dir.join(relative);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Bundle CSS files (resolves @imports), copy others directly
        if src.extension().is_some_and(|ext| ext == "css") {
            let original_size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
            let bundled = bundle_css(src).map_err(Error::CssBundle)?;
            fs::write(&dest, &bundled).map_err(|e| Error::WriteFile {
                path: dest.clone(),
                source: e,
            })?;
            eprintln!(
                "bundling: {} → {} ({} → {} bytes)",
                src.display(),
                dest.display(),
                original_size,
                bundled.len()
            );
        } else {
            fs::copy(src, &dest).map_err(|e| Error::WriteFile {
                path: dest.clone(),
                source: e,
            })?;
            eprintln!("copying: {} → {}", src.display(), dest.display());
        }
    }

    Ok(())
}
