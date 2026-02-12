---
title: sukr
description: Minimal static site compiler — suckless, Rust, zero JS
---

# sukr

**sukr** transforms Markdown into high-performance static HTML. No bloated runtimes, no client-side JavaScript, just clean output.

## Why sukr?

Most static site generators punt rich content to the browser. sukr doesn't.

- **Tree-sitter highlighting** — Proper parsing, not regex. Supports language injection (Nix→Bash, HTML→JS/CSS).
- **Build-time math** — KaTeX renders LaTeX to static HTML. No 300KB JavaScript bundle.
- **Build-time diagrams** — Mermaid compiles to inline SVG. Diagrams load instantly.
- **Flexible templates** — Runtime Tera templates, no recompilation needed.
- **Monorepo-ready** — Multiple sites via `-c` config flag.

Ready to try it? Start with the [Getting Started](getting-started.html) guide.

## Learn More

- [Getting Started](getting-started.html) — install sukr and build your first site
- [Deployment](deployment.html) — deploy your site to GitHub Pages, Netlify, or any static host
- [Configuration](configuration.html) — `site.toml` reference and CLI options
- [Content Organization](content-organization.html) — how directories map to site structure
- [Architecture](architecture.html) — how sukr works under the hood

Browse the **Features** section in the sidebar for syntax highlighting, math, diagrams, and more.
