# CHARTER: sukr 1.0 Stable Release

<!--
  Strategic framing document for the sukr 1.0 milestone.
  Defines purpose, success criteria, and ordered workstreams.

  See: workflows/charter.md for the full protocol specification.
-->

## Purpose

sukr exists because static site generators make the wrong tradeoff: they ship complexity to the browser. Rich content (syntax highlighting, math, diagrams) gets rendered client-side via heavy JavaScript bundles — KaTeX alone is 300KB. This punishes readers on slow connections, breaks accessibility, and violates the principle that a static site should be _static_.

The people who care about this are developers and technical writers who want fast, correct, zero-JS output from Markdown without giving up features that matter: proper Tree-sitter highlighting with language injection, LaTeX math, Mermaid diagrams, and customizable templates.

sukr is a single Rust binary that intercepts pulldown-cmark's event stream and compiles rich content to static HTML/SVG at build time. The output directory is deployable anywhere — no runtime, no dependencies, no JavaScript.

## North Star

A technical writer clones a repo, runs `sukr`, and gets a fully-rendered static site with syntax highlighting, math, and diagrams — all as server-rendered HTML/SVG. No npm. No JS payload. No CDN dependencies. The binary is hermetic (Nix-built), the output is auditable (`view-source:` shows exactly what's served), and the tool is stable enough to depend on without chasing breaking changes.

The measure isn't feature count — it's _absence_. Absence of JS bundles, absence of runtime dependencies, absence of unnecessary API churn. sukr succeeds when it disappears: you think about your content, not your tooling.

## Workstreams

<!--
  Ordered by dependency and risk. Infrastructure first, then
  user-facing stability, then distribution.
-->

1. **API Stabilization** — Evaluate which features belong in 1.0 (candidates: taxonomies/tags, i18n, others surfaced during sketching), then lock the public contract: `site.toml` schema, frontmatter fields, template variables, CLI flags, content directory conventions. Includes a dependency health audit (git deps, alpha deps, local patches) since these affect whether the stable surface is reproducibly buildable. The sketch must answer "what's in?" before locking "what's stable?" — and may spawn additional feature workstreams if the answer introduces implementation work
   - Spawns: `.sketches/api-stabilization.md` (and potentially feature-specific sketches)
   - Status: Complete — see `docs/plans/api-stabilization.md`

2. **Error Quality** — Every error message tells the user what failed, why, and what to do about it. No panics outside `main()`, no opaque messages
   - Spawns: `.sketches/error-quality.md`
   - Status: Not Started

3. **Test Coverage** — Sufficient tests to protect the stabilized API surface. Content transformation correctness, template resolution, frontmatter parsing, edge cases
   - Spawns: `.sketches/test-coverage.md`
   - Status: Not Started

4. **Distribution** — Users can install sukr without building from source. Nix flake (`nix build`), `cargo install`, and prebuilt binaries for common platforms. Includes CI/CD pipeline for automated testing and release builds, and a changelog convention for communicating changes
   - Spawns: `.sketches/distribution.md`
   - Status: Not Started

## Non-Goals

- **Plugin system.** sukr's value is in being a single binary with known behavior. A plugin API introduces version coupling, runtime loading, and support burden disproportionate to the user base. If someone needs extensibility, Tera templates already provide customization at the layout layer. Interceptor-level extensibility is a post-1.0 concern, if ever.

- **Incremental builds.** Full rebuilds are fast enough for sites in the hundreds-of-pages range. Incremental builds add cache invalidation complexity (content depends on templates depends on config depends on CSS) that isn't justified until there's measured pain. Premature.

- **Additional output formats.** sukr outputs HTML. Not PDF, not EPUB, not Gemini. Each format requires its own rendering pipeline, its own testing, its own edge cases. This is a static _site_ compiler, not a document converter.

- **Built-in development server.** `python -m http.server` or any static file server works. Embedding a server adds dependencies, complicates the binary, and solves a problem that's already solved. Live reload is a post-1.0 luxury.

- **Migration tooling.** No importers for Hugo, Zola, Jekyll, etc. The content format (Markdown + TOML frontmatter) is standard enough that migration is a text-editing problem, not a tooling problem.

## Appetite

A focused sprint — 3–5 sketch→plan→core cycles over 2–4 weeks. sukr already works; the gap between 0.1 and 1.0 is stabilization, not new features. Most of the work is auditing what exists, locking it down, and filling quality gaps (errors, tests, distribution).

If workstream scope inflates beyond what's needed to declare "this API won't break," that's a signal to descope, not to extend the timeline.

## References

<!--
  This section grows as workstreams progress through the pipeline.
-->

- Plans: `docs/plans/[topic].md`
- Sketches: `.sketches/[topic].md`
- Existing: `docs/plans/documentation-overhaul.md` (Complete)
