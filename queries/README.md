# Syntax Highlighting Queries

This directory contains Tree-sitter query files for syntax highlighting.

## Attribution

These queries are derived from the [Helix editor](https://github.com/helix-editor/helix) project.

Helix is licensed under the Mozilla Public License 2.0 (MPL-2.0).

We are grateful to the Helix maintainers and contributors for their excellent work on comprehensive, well-tested Tree-sitter queries.

## Structure

Each language has its own subdirectory containing:

- `highlights.scm` — Syntax highlighting queries
- `injections.scm` — Language injection queries (e.g., bash in Nix strings)
- `locals.scm` — Local variable scoping (where applicable)

## Supported Languages

- Bash / Shell
- C
- CSS
- Go
- HTML
- JavaScript
- JSON
- Markdown
- Nix
- Python
- Rust
- TOML
- TypeScript
- Slint
- YAML
