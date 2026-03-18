<p align="center">
  <img src="docs/static/logo.png" alt="sukr logo" width="128" />
</p>

# sukr

**Minimal static site compiler — suckless, Rust, zero JS.**

sukr transforms Markdown content into high-performance static HTML. No bloated runtimes, no client-side JavaScript, just clean output.

## Why sukr?

Most static site generators punt rich content to the browser. sukr doesn't.

- **Tree-sitter syntax highlighting** — Proper parsing, not regex. Supports language injection (Nix shells, HTML scripts).
- **Build-time math** — LaTeX renders to native MathML. No 300KB JavaScript bundle.
- **Build-time diagrams** — Mermaid compiles to inline SVG. Diagrams load instantly.
- **Tera templates** — Customize layouts without recompiling.
- **Monorepo support** — Multiple sites via `-c` flag.

See the [full feature comparison](https://sukr.io/comparison.html) with Zola, Hugo, and Eleventy.

## Quick Start

```bash
cargo build --release
sukr                         # Build with ./site.toml
sukr -c docs/site.toml       # Custom config (monorepo)
```

See the [Getting Started guide](https://sukr.io/getting-started.html) for installation and first-site setup, [Configuration](https://sukr.io/configuration.html) for `site.toml` options, and [Content Organization](https://sukr.io/content-organization.html) for directory layout.

## Security

sukr processes content at **build time only** — there is no runtime attack surface.

- **Untrusted:** Markdown content, frontmatter, third-party templates
- **Trusted:** The compiled sukr binary, Tree-sitter grammars

Raw HTML in Markdown is passed through per CommonMark spec — review content from untrusted sources before building. Templates use Tera's auto-escaping by default.

For deployment security (CSP headers, platform configs), see the [Security docs](https://sukr.io/security.html).

## Documentation

Full documentation at [sukr.io](https://sukr.io) (built with sukr).

## License

MIT
