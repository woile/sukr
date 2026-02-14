+++
title = "Content Organization"
description = "How the filesystem maps to your site structure"
weight = 2
+++

sukr builds your site structure from your `content/` directory. No routing config needed — the filesystem _is_ the config.

## The Rule

```text
content/foo/bar.md  →  public/foo/bar.html
content/about.md    →  public/about.html
content/_index.md   →  public/index.html
```

That's it. Paths mirror exactly, with `.md` becoming `.html`.

## Directory Layout

```text
content/
├── _index.md           # Homepage (required)
├── about.md            # → /about.html
├── contact.md          # → /contact.html
├── blog/               # Section directory
│   ├── _index.md       # → /blog/index.html (section index)
│   ├── first-post.md   # → /blog/first-post.html
│   └── second-post.md  # → /blog/second-post.html
└── projects/
    ├── _index.md       # → /projects/index.html
    └── my-app.md       # → /projects/my-app.html
```

## What Makes a Section

A section is any directory containing `_index.md`. This file:

1. Provides metadata for the section (title, description)
2. Triggers section listing behavior
3. Appears in the navigation

Directories without `_index.md` are ignored.

## Section Discovery

sukr automatically discovers sections during the build:

1. Scans `content/` for directories containing `_index.md`
2. Collects all `.md` files in that directory (excluding `_index.md`)
3. Renders the section index template with the collected items
4. Renders individual content pages (for blog-type sections)

The **section type** determines which template renders the index. It resolves in order:

1. **Frontmatter override** — `section_type = "blog"` in the section's `_index.md`
2. **Directory name** — `content/blog/` becomes type `blog`

For the full section type reference (built-in types, frontmatter fields, and template dispatch), see [Sections](features/sections.html).

## Navigation Generation

Navigation builds automatically from:

- **Top-level `.md` files** (except `_index.md`) → page links
- **Directories with `_index.md`** → section links

Items sort by `weight` in frontmatter (lower first), then alphabetically.

```toml
+++
title = "Blog"
weight = 10  # Appears before items with weight > 10
+++
```

### Hierarchical Navigation

When `nav.nested = true` in your config, section children appear as nested sub-items:

```text
Features           ← Section link
  ├─ Templates     ← Child page
  ├─ Sections      ← Child page
  └─ Highlighting  ← Child page
Getting Started    ← Top-level page
```

Child pages inherit their parent section's position in the nav tree. Within a section, children sort by weight then alphabetically.

Without nested navigation (the default), only top-level items appear in the nav.

## URL Examples

| Source Path              | Output Path              | URL                |
| ------------------------ | ------------------------ | ------------------ |
| `content/_index.md`      | `public/index.html`      | `/`                |
| `content/about.md`       | `public/about.html`      | `/about.html`      |
| `content/blog/_index.md` | `public/blog/index.html` | `/blog/`           |
| `content/blog/hello.md`  | `public/blog/hello.html` | `/blog/hello.html` |

## Key Points

- No config files for routing
- Directory names become URL segments
- `_index.md` = section index, not a regular page
- Flat output structure (no nested `index.html` per page)
