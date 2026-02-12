---
title: Configuration
description: Complete reference for site.toml configuration
weight: 3
---

sukr is configured via `site.toml`. All settings have sensible defaults.

## Basic Configuration

```toml
title    = "My Site"
author   = "Your Name"
base_url = "https://example.com"
```

| Field      | Required | Description                      |
| ---------- | -------- | -------------------------------- |
| `title`    | Yes      | Site title (used in page titles) |
| `author`   | Yes      | Author name (used in feeds)      |
| `base_url` | Yes      | Canonical URL for the site       |

## Path Configuration

All paths are optional. Default values shown:

```toml
[paths]
content   = "content"    # Markdown source files
output    = "public"     # Generated HTML output
static    = "static"     # Static assets (copied as-is)
templates = "templates"  # Tera template files
```

Paths are resolved **relative to the config file location**. This enables monorepo setups:

```bash
# Build site from subdirectory
sukr -c sites/blog/site.toml
# Paths resolve relative to sites/blog/
```

## Navigation Configuration

Control how navigation menus are generated:

```toml
[nav]
nested = false  # Show child pages under sections
toc    = false  # Enable table of contents globally
```

| Field    | Default | Description                                         |
| -------- | ------- | --------------------------------------------------- |
| `nested` | `false` | Display section children as nested sub-menus        |
| `toc`    | `false` | Show heading anchors in sidebar (table of contents) |

When `nested = true`, section pages appear as indented sub-items under their parent section. When `toc = true`, h2-h6 headings are extracted and displayed as anchor links in the sidebar.

Both settings can be overridden per-page via frontmatter.

## CLI Options

```bash
sukr                           # Use ./site.toml
sukr -c path/to/site.toml      # Custom config
sukr --config path/to/site.toml
sukr -h, --help                # Show help
```

## Frontmatter

Each Markdown file can have YAML frontmatter:

```yaml
---
title: Page Title
description: Optional description
date: 2024-01-15 # For blog posts
weight: 10 # Sort order (lower = first)
nav_label: Short Name # Override nav display
section_type: blog # Override section template
template: custom # Override page template
toc: true # Override global TOC setting
link_to: https://... # External link (for project cards)
---
```

### Frontmatter Fields

| Field          | Type    | Default        | Description                                    |
| -------------- | ------- | -------------- | ---------------------------------------------- |
| `title`        | string  | _(required)_   | Page title                                     |
| `description`  | string  | _(none)_       | Meta description                               |
| `date`         | string  | _(none)_       | Publication date (YYYY-MM-DD)                  |
| `weight`       | integer | `50`           | Sort order (lower = first)                     |
| `nav_label`    | string  | title          | Override navigation label                      |
| `section_type` | string  | directory name | Template dispatch (e.g., "blog", "projects")   |
| `template`     | string  | _(none)_       | Custom template name                           |
| `toc`          | boolean | global setting | Enable/disable table of contents for this page |
| `link_to`      | string  | _(none)_       | External URL (renders as link instead of page) |
| `tags`         | list    | `[]`           | Tags for categorization                        |

### Section Types

The `section_type` field determines which template is used for section indexes:

- `blog` → `templates/section/blog.html`
- `projects` → `templates/section/projects.html`
- _(any other)_ → `templates/section/default.html`

If not specified, sukr uses the directory name as the section type.

## See Also

- [Getting Started](getting-started.html) — install sukr and build your first site
- [Content Organization](content-organization.html) — how directories map to site structure
- [Templates](features/templates.html) — template directory structure and customization
