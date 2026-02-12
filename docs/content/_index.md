---
title: sukr
description: Minimal static site compiler — suckless, Rust, zero JS
---

# Welcome to sukr

**sukr** transforms Markdown into high-performance static HTML. No bloated runtimes, no client-side JavaScript, just clean output.

## Why sukr?

Most static site generators punt rich content to the browser. sukr doesn't.

- **Tree-sitter highlighting** — Proper parsing, not regex. Supports language injection (Nix→Bash, HTML→JS/CSS).
- **Build-time math** — KaTeX renders LaTeX to static HTML. No 300KB JavaScript bundle.
- **Build-time diagrams** — Mermaid compiles to inline SVG. Diagrams load instantly.
- **Flexible templates** — Runtime Tera templates, no recompilation needed.
- **Monorepo-ready** — Multiple sites via `-c` config flag.

## Quick Start

```bash
# Install
git clone https://github.com/nrdxp/sukr
cd sukr
cargo install --path .

# Create site structure
mkdir -p content templates static
echo 'title = "My Site"' > site.toml
echo 'author = "Me"' >> site.toml
echo 'base_url = "https://example.com"' >> site.toml

# Build
sukr
```

## Documentation

Browse the sidebar for detailed documentation on all features.
