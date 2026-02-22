# PLAN: Formal Model Alignment

<!--
  Aligns the sukr implementation with the categorical model
  defined in docs/models/sukr-compiler.md.

  Sketch: .sketches/2026-02-21-sukr-formal-model.md
-->

## Goal

Restructure the sukr compiler implementation to align with the formal
categorical model (Source → Content → Output via Parse and Compile functors).
The current implementation broadly matches the model's intent but has structural
gaps: the Content category lacks explicit types for several model objects, the
Parse and Compile phases are interleaved rather than cleanly separated, and
cross-dependency validation (links, tags) happens in the wrong phase or not at
all. This plan systematically closes every gap, ordered to preserve
independently valuable increments.

## Constraints

- Rust 2024 edition, suckless principles (no new dependencies unless essential)
- Pre-1.0 (`0.x.x`): breaking internal API changes are acceptable
- Zero-JS output invariant must be maintained
- All existing tests must continue to pass (or be updated to match new types)
- Each phase must compile and pass `cargo test` before the next begins

## Decisions

| Decision                      | Choice                                                | Rationale                                                                                                                                                                                                                                                                                                                |
| :---------------------------- | :---------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ContentBlock representation   | Explicit enum in `content.rs`                         | Model defines a coproduct; current code uses pulldown-cmark events directly in render.rs. A typed enum makes the interception pattern explicit and testable.                                                                                                                                                             |
| Parse/Compile separation      | Clean module boundary via Content types as interface  | Model requires Parse (S→C) and Compile (C→O) to be distinct functors with Content as the interface category. Current `main.rs::run` interleaves discovery with rendering.                                                                                                                                                |
| Tag as first-class type       | Newtype `Tag(String)` in content.rs                   | Model gives Tag its own object status. Current implementation treats tags as bare `Vec<String>` in frontmatter. Newtype prevents accidental conflation with arbitrary strings.                                                                                                                                           |
| Link extraction phase         | During Parse (content discovery), not during Render   | Model says `references` morphisms belong in Category C, constructed by Parse. Currently links are only discovered during rendering.                                                                                                                                                                                      |
| Frontmatter delimiter         | Keep `+++` (TOML) as-is                               | Model says "TOML frontmatter" which is already the case. The `+++` delimiters are the implementation convention; no change needed.                                                                                                                                                                                       |
| ContentBlock granularity      | Start with 7 variants from model, extend later        | Code, Math, Diagram, Heading, Text, Link, Image. Additional variants (Table, List, etc.) deferred — a natural extension point once the coproduct exists.                                                                                                                                                                 |
| Error phase partitioning      | `ParseError` + `CompileError` enums                   | Model defines Parse and Compile as distinct partial functors with distinct failure modes. A single `Error` enum conflates them. Phase-split errors make the functor boundary visible in Rust's type system.                                                                                                              |
| Output path computation       | Store as field, not method                            | Model says output path is uniquely determined by `section.slug + page.slug`. Currently recomputed via `Content::output_path(&self, content_root)` at every call site. Storing it as a `PathBuf` field computed once during Parse eliminates the `content_root` argument threading and makes the invariant a stored fact. |
| SectionType                   | `enum SectionType { Blog, Projects, Custom(String) }` | Model + implementation both dispatch on section type for sorting and template selection. Currently `Option<String>`. An enum makes dispatch exhaustive at compile time — adding a new section type forces handling everywhere.                                                                                           |
| Nav derivation                | Derive from parsed data, not filesystem               | Nav tree is a function of sections + pages + weights (model invariant 2). Current `discover_nav()` re-reads the filesystem. `derive_nav()` from `SiteManifest` eliminates inconsistency by construction.                                                                                                                 |
| ValidatedRef for links        | Parse-don't-validate newtype                          | Inter-page link targets should be validated during Parse. A `ValidatedRef` newtype, constructible only after validation, ensures Compile code can never receive an unvalidated reference.                                                                                                                                |
| Sorted-by-construction        | `BTreeSet`/`BTreeMap` directly — no wrappers          | Nav: `BTreeSet<NavItem>` with `impl Ord`. Section items: `BTreeMap<SortKey, Content>` where `SortKey` is a plain key type implementing `Ord`, constructed from `SectionType` + item metadata. No ad-hoc collection wrappers.                                                                                             |
| Out-of-model code integration | Respect functor boundaries                            | Static asset copying, alias generation, CSS bundling are out of model scope but must still respect Parse/Compile phase boundaries, use proper error types (`CompileError`), and take typed inputs from Category C.                                                                                                       |
| ContentKind type split        | Defer                                                 | Model defines distinct objects (Page, Section, Homepage). Splitting into distinct Rust types has HIGH value but a LARGE blast radius. Defer to Phase 4 evaluation after Phases 1-3 reveal whether runtime `ContentKind` checking causes friction.                                                                        |

## Risks & Assumptions

| Risk / Assumption                                                                                      | Severity | Status      | Mitigation / Evidence                                                                                                                                                                          |
| :----------------------------------------------------------------------------------------------------- | :------- | :---------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ContentBlock enum may not cover all pulldown-cmark events cleanly                                      | MEDIUM   | Unvalidated | Spike needed: map all pulldown-cmark events to ContentBlock variants. Unmapped events get a `Raw(String)` fallback variant during transition.                                                  |
| Link extraction during Parse requires a two-pass approach (frontmatter parse, then markdown link scan) | MEDIUM   | Validated   | pulldown-cmark can be run in a lightweight mode to extract links without full rendering. The body is already stored as a String in `Content`.                                                  |
| Large render.rs refactor may temporarily break Mermaid/KaTeX/Tree-sitter interception                  | MEDIUM   | Unvalidated | Mitigation: Phase 3 (Compile) preserves existing render logic as the catamorphism's per-variant functions. No rendering logic is deleted, only restructured.                                   |
| Phase ordering dependency: ContentBlock enum (Phase 2) must exist before Compile refactor (Phase 3)    | LOW      | Validated   | Phases are ordered to respect this dependency.                                                                                                                                                 |
| Tag newtype may require downstream changes in template_engine.rs and sitemap.rs                        | LOW      | Validated   | Tag implements `Display`, `AsRef<str>`, and `Serialize`. Template engine and sitemap code that uses `String` will work via `.as_ref()`.                                                        |
| Error split increases error handling verbosity at the `main.rs` boundary                               | LOW      | Unvalidated | Mitigation: `main.rs` can use `From<ParseError> for Error` and `From<CompileError> for Error` to unify at the top-level `run()` boundary while keeping phase errors distinct in module APIs.   |
| SectionType enum `Custom(String)` fallback may not be extensible                                       | LOW      | Validated   | The `Custom(String)` variant preserves current behavior for user-defined section types. Known types (Blog, Projects) get exhaustive matching; unknown types fall through to a default handler. |

## Open Questions

- **Draft page semantics:** The model says Parse "simply does not create objects for drafts." The current implementation checks `draft == true` and skips during `collect_items()` and `discover_pages()`. This is already aligned — drafts are filtered during Parse. No change needed. **Resolved.**
- **Alias morphism formalization:** The model notes aliases aren't explicitly modeled as morphisms in C. For now, aliases are handled in Compile (output generation) which is acceptable. Defer formalization until aliases become more complex. **Deferred.**
- **Incremental compilation:** The model acknowledges this is a future concern. Explicitly out of scope. **Deferred.**

## Scope

### In Scope

- Introduce `ContentBlock` enum (the coproduct from the model)
- Introduce `Tag` newtype
- Parse markdown body into `Vec<ContentBlock>` during content discovery
- Extract inter-page links (references morphisms) during Parse
- Validate reference integrity (broken links) before Compile begins
- Clean separation of Parse and Compile module boundaries
- Restructure `render.rs` as a catamorphism over `ContentBlock`
- Comprehensive tests for each new type and the Parse→Compile boundary
- Update `SiteManifest` to serve as the complete Category C representation
- Update error types to cover model's functor failure modes
- **Out-of-model code integration:** Static asset copying, alias generation, CSS bundling, feed/sitemap generation — refactored to use typed error types (`CompileError`) and consume typed inputs from Category C. Execution logic unchanged; error model and phase positioning refined.
- **Template engine hardcoding:** Extract/configure hardcoded template names; unify template override pattern across all render methods
- **Hardcoded convention extraction:** Magic strings, output filenames, weight defaults → named constants or configuration

### Out of Scope

- **Incremental compilation** — model acknowledges as future concern, explicitly deferred
- **New output formats** — model doesn't require them
- **Nested section support** — currently only one level deep; model doesn't require depth changes
- **Alias formalization as morphisms in C** — aliases are handled during Compile (output generation) which is acceptable; their _integration_ with typed inputs and `CompileError` IS in scope, but formalizing them as first-class morphisms in the Content category is deferred
- **Tera engine replacement** — template _engine_ is out of scope; only hardcoded template name extraction and render method consistency are in scope
- **Static asset pipeline redesign** — the copy-all-files behavior stays as-is; only error typing (`CompileError::StaticAssetCopy`) and CSS bundling error typing are in scope

## Pre-Identified Cruft

Items catalogued during pre-execution audit. Each is assigned to a phase for
in-context cleanup rather than ad-hoc discovery.

| #   | Item                                               | Location                                                                                                                                          | Problem                                                                                                                                        | Phase                                                                                                      |
| :-- | :------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------ | :--------------------------------------------------------------------------------------------------------------------------------------------- | :--------------------------------------------------------------------------------------------------------- |
| C1  | `section.collect_items()` called 6×                | `run()`, `collect_tags()`, `generate_aliases()`, `sitemap.rs`, `discover_inner()`, `discover_nav()`                                               | Each call re-reads the filesystem. Section items should be collected once during Parse and stored on `Section`.                                | **Phase 2** — items become a field on `Section` (`BTreeMap<SortKey, Content>`), populated during discovery |
| C2  | `process_pages()` ignores `manifest.pages`         | `main.rs:266`                                                                                                                                     | Re-reads filesystem for standalone pages that `SiteManifest::discover` already collected.                                                      | **Phase 4** — replace with iteration over `manifest.pages`                                                 |
| C3  | `discover_nav()` re-reads FS                       | `content.rs:201`                                                                                                                                  | Re-reads content directory and re-parses content files that `discover_sections` and `discover_pages` already processed.                        | **Phase 2** — replaced by `derive_nav()` from parsed data                                                  |
| C4  | `content_dir` threaded through 8+ functions        | `run()`, `write_output()`, `ContentContext::from_content()`, `collect_tags()`, `generate_aliases()`, `generate_feed()`, `generate_sitemap_file()` | Sole purpose: pass to `Content::output_path(content_dir)`. Becomes unnecessary when `output_path` is a stored field.                           | **Phase 1** — `output_path` becomes a field; Phase 4 — remove the parameter from all downstream functions  |
| C5  | `write_output()` takes `content_dir` arg           | `main.rs:431`                                                                                                                                     | Only needed for `content.output_path(content_dir)`.                                                                                            | **Phase 4** — simplify to `write_output(output_dir, content, html)`                                        |
| C6  | `ContentContext::from_content` takes `content_dir` | `template_engine.rs:238`                                                                                                                          | Only needed to call `content.output_path(content_dir)`.                                                                                        | **Phase 4** — simplify to `ContentContext::from_content(content, config)`                                  |
| C7  | `DEFAULT_WEIGHT_HIGH` constant                     | `content.rs:13`, used only in `main.rs:128-129`                                                                                                   | Used in exactly one match arm (projects sort). If `SortKey` handles weight defaults, this constant becomes dead.                               | **Phase 1** — absorbed into `SortKey::WeightTitle` construction logic                                      |
| C8  | `DEFAULT_WEIGHT` constant                          | `content.rs:10`, used in `discover_nav()` and `run()`                                                                                             | Same pattern as C7 — weight default logic should live where `SortKey`/`NavItem` is constructed, not as a module-level constant.                | **Phase 1/2** — absorbed into `NavItem` or `SortKey` construction                                          |
| C9  | `Section::collect_items()` method                  | `content.rs:304`                                                                                                                                  | Becomes dead once section items are stored as a field (see C1). The method re-reads the filesystem, which violates the "Parse once" principle. | **Phase 2** — remove entirely; items are a field populated during discovery                                |

## Hardcoded Assumptions Inventory

Items that prevent sukr from being a truly generic static site compiler.
Each should be abstracted to configuration, type-level dispatch, or at
minimum documented as an explicit convention rather than buried in code.

| #   | Hardcoded Value                                     | Location(s)                                        | Should Be                                                                                                                                                                                                                   | Phase                                                                                                           |
| :-- | :-------------------------------------------------- | :------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------- |
| H1  | `"blog"` magic string for sort dispatch             | `main.rs:119`, `content.rs:321`, `content.rs:459`  | `SectionType::Blog` enum variant — dispatch via `match` on typed enum, not string comparison                                                                                                                                | **Phase 1** — `SectionType` enum replaces all string comparisons                                                |
| H2  | `"projects"` magic string for sort dispatch         | `main.rs:123`, `content.rs:322`                    | `SectionType::Projects` enum variant                                                                                                                                                                                        | **Phase 1** — same as H1                                                                                        |
| H3  | `"content/default.html"` template fallback          | `template_engine.rs:76`                            | Configurable default content template in `site.toml`, or at minimum a named constant                                                                                                                                        | **Phase 4** — evaluate whether template names should be configurable                                            |
| H4  | `"section/default.html"` template fallback          | `template_engine.rs:105`                           | Same as H3                                                                                                                                                                                                                  | **Phase 4**                                                                                                     |
| H5  | `"tags/default.html"` template name                 | `template_engine.rs:133`                           | Same as H3                                                                                                                                                                                                                  | **Phase 4**                                                                                                     |
| H6  | `"page.html"` template name                         | `template_engine.rs:59`                            | Same as H3                                                                                                                                                                                                                  | **Phase 4**                                                                                                     |
| H7  | `DEFAULT_WEIGHT = 50`                               | `content.rs:10`                                    | Configurable in `site.toml` under `[defaults]`, or as a constant with a clear name tied to the type system                                                                                                                  | **Phase 1** — absorbed into `SortKey` / `NavItem` construction, value becomes a type-level association          |
| H8  | `DEFAULT_WEIGHT_HIGH = 99`                          | `content.rs:13`                                    | Same as H7 — only used for projects sort                                                                                                                                                                                    | **Phase 1** — absorbed into `SortKey::WeightTitle`                                                              |
| H9  | `"_index.md"` as section index convention           | `content.rs:217,235,317,349,400,439` (6 locations) | Named constant `const SECTION_INDEX: &str = "_index.md"` — convention is fine, but should not be a string literal in 6 places                                                                                               | **Phase 2** — extract to constant                                                                               |
| H10 | `"_404.md"` as 404 page convention                  | `content.rs:217,400,443`                           | Named constant `const PAGE_404: &str = "_404.md"`                                                                                                                                                                           | **Phase 2** — extract to constant                                                                               |
| H11 | Section type → `ContentKind` mapping                | `content.rs:320-323`                               | `SectionType` enum dispatch — once `SectionType` exists, this `match` on strings becomes a typed dispatch. But: should `ContentKind` even exist once `SectionType` does the dispatching? See deferred ContentKind decision. | **Phase 1** — partially resolved by `SectionType`; Phase 4 — evaluate if `ContentKind` is still needed          |
| H12 | `"Tagged: {}"` format string for tag page titles    | `template_engine.rs:130`                           | Configurable in `site.toml` or template — this is a display concern that shouldn't be in Rust code                                                                                                                          | **Phase 4** — move to template or config                                                                        |
| H13 | `"feed.xml"` output filename                        | `main.rs:229`                                      | Named constant or configurable in `[feed]` section of `site.toml`                                                                                                                                                           | **Phase 4** — extract to constant                                                                               |
| H14 | `"sitemap.xml"` output filename                     | `main.rs:251`                                      | Named constant or configurable in `[sitemap]` section of `site.toml`                                                                                                                                                        | **Phase 4** — extract to constant                                                                               |
| H15 | `"404.html"` output filename                        | `main.rs:349`                                      | Named constant — pairs with H10 (`_404.md` input)                                                                                                                                                                           | **Phase 4** — extract to constant                                                                               |
| H16 | `"index.html"` as homepage output                   | `main.rs:320`, `content.rs:131`                    | Named constant — conventional but appears in multiple locations                                                                                                                                                             | **Phase 4** — extract to constant                                                                               |
| H17 | `render_page` lacks `frontmatter.template` override | `template_engine.rs:59`                            | `render_content` supports `frontmatter.template` override (line 72-76), but `render_page` hardcodes `"page.html"`. Inconsistent — for a generic compiler, all content should support template override via frontmatter.     | **Phase 4** — unify template override pattern: `frontmatter.template.unwrap_or(default)` for all render methods |

## Phases

1. **Phase 1: Content Category Types** — Establish the type system for Category C

   **New types:**
   - [x] Define `ContentBlock` enum with 7+1 variants: `Code`, `Math`, `Diagram`, `Heading`, `Text`, `Link`, `Image`, `Raw` (fallback)
   - [x] Define `Tag` newtype with `Display`, `AsRef<str>`, `Serialize`, `Deserialize`, `Eq`, `Hash`, `Ord`
   - [x] Define `SectionType` enum: `Blog`, `Projects`, `Custom(String)` with `Display`, `Serialize`, `Deserialize` — **resolves H1, H2, H11**
   - [x] Define `SortKey` enum: `DateDesc(NaiveDate)`, `WeightTitle(i64, String)` with appropriate `Ord` — a plain key type, no collection wrapper — **absorbs C7 (`DEFAULT_WEIGHT_HIGH`), C8 (`DEFAULT_WEIGHT`) into construction logic, resolves H7, H8**
   - [x] `impl Ord for NavItem` with `(weight, label)` ordering; `impl PartialOrd`, `Eq`, `PartialEq` (derive or manual)
   - [x] Define `LinkTarget` type (relative path or URL with source location for error reporting)

   **Struct updates:**
   - [x] Update `Frontmatter.tags` from `Vec<String>` to `Vec<Tag>`
   - [x] Update `Frontmatter.section_type` from `Option<String>` to `Option<SectionType>`
   - [x] Update `Content` struct: add `blocks: Vec<ContentBlock>`, `links: Vec<LinkTarget>`, `output_path: PathBuf`
   - [x] Remove `Content::output_path()` method, replace all call sites with field access — **resolves C4 (field side; parameter pruning in Phase 4)**
   - [x] Update `Section`: add `items: Vec<Content>` field — items collected and sorted at construction time — **resolves C1 (sorted-by-construction)**
   - [x] Update `discover_sections` and `discover_pages` to return sorted-by-construction collections (no post-hoc `sort_by`)

   **Downstream consumers:**
   - [x] Update all consumers of `Frontmatter.tags`: `collect_tags`, `write_tag_pages`, `FrontmatterContext::new`, `sitemap.rs`
   - [x] Update all consumers of `section_type`: sort dispatch in `run()` , template resolution in `template_engine.rs` — replace `match section.section_type.as_str()` with `match section.section_type`
   - [x] Remove `DEFAULT_WEIGHT` and `DEFAULT_WEIGHT_HIGH` constants — **resolves C7, C8**

   **Cruft + verification:**
   - [x] **Cruft audit:** Removed `Section.path`, `Section.content_root` (no readers after `collect_items` removal). `Tag::new`/`as_str` and `SortKey::for_content` retained (test-only usage). Pre-1.0 = no backwards compat tax.
   - [x] All existing tests pass, new unit tests for `ContentBlock`, `Tag`, `SectionType`, `SortKey`, `LinkTarget`

2. **Phase 2: Parse Functor** — Content discovery produces fully-typed Category C objects

   **Block parsing + link extraction:**
   - [x] Implement `parse_blocks(markdown: &str) -> Vec<ContentBlock>` in `content.rs`
   - [x] Implement `extract_links(blocks: &[ContentBlock]) -> Vec<LinkTarget>` to discover inter-page references
   - [x] Update `Content::from_path` to populate `blocks`, `links`, and `output_path` fields

   **Section items as stored field (resolves C1, C9):**
   - [x] Populate `Section.items` during `discover_sections` — sorted `Vec<Content>` at construction — **resolves C1** _(completed Phase 1 C4)_
   - [x] Remove `Section::collect_items()` method entirely — **resolves C9** _(completed Phase 1 C4)_
   - [x] Update all 6 sites that called `section.collect_items()` to use `section.items` directly: `run()`, `collect_tags()`, `generate_aliases()`, `sitemap.rs`, `discover_inner()`, `discover_nav()` _(completed Phase 1 C4)_

   **Nav derivation (resolves C3):**
   - [x] Replace `discover_nav()` with `derive_nav()` that builds nav from already-parsed `sections` and `pages`, returning `Vec<NavItem>` sorted at construction — **resolves C3**
   - [x] Kept `SiteManifest.nav` as `Vec<NavItem>` — BTreeSet rejected due to NavItem's lossy PartialEq _(deviation from plan)_
   - [x] `template_engine.rs::base_context` already accepts `&Vec<NavItem>` — no change needed

   **Hardcoded conventions → named constants (resolves H9, H10):**
   - [x] Extract `"_index.md"` to `const SECTION_INDEX: &str` — used in 6 locations — **resolves H9**
   - [x] Extract `"_404.md"` to `const PAGE_404: &str` — used in 3 locations — **resolves H10**

   **Reference validation:**
   - [x] Add reference integrity validation to `SiteManifest::discover`: validate every `LinkTarget` pointing to an internal path — broken links produce non-fatal warnings
   - [ ] ~~Define `ValidatedRef` newtype~~ — deferred: no downstream consumer yet
   - [x] Add `Error::BrokenLink { source_page, target, line }` error variant

   **Cruft + verification:**
   - [x] **Cruft audit:** Removed `discover_nav()` after `derive_nav()` migration (8 tests removed, 1 added) _(Note: `DEFAULT_WEIGHT`/`DEFAULT_WEIGHT_HIGH` already removed in Phase 1 C3, `collect_items()` already removed in Phase 1 C4)_
   - [x] Tests: broken link detection, valid link pass-through, external link ignored, normalize_link_url

3. **Phase 3: Compile Functor** — Rendering dispatches intercepted blocks, passes through Prose

   **Model refinement (pre-execution):**
   - [x] Remove `ContentBlock::Text` — absorbed into `Prose` (sukr doesn't intercept plain text)
   - [x] Rename `ContentBlock::Raw` → `ContentBlock::Prose` (structural honesty: standard rendering, not "raw")
   - [x] Remove `ContentBlock::Link` and `ContentBlock::Image` from the coproduct — reference extraction is a Parse side-channel via `extract_links`, not a block type. Rework `extract_links` to operate on pulldown-cmark events directly during `parse_blocks`.

   **Catamorphism:**
   - [x] Add `render_blocks(blocks: &[ContentBlock]) -> (String, Vec<Anchor>)` to `render.rs`
   - [x] Intercepted variant dispatch:
     - [x] `Code` → `highlight_code` (existing)
     - [x] `Math` → `crate::math::render_math` (existing)
     - [x] `Diagram` → `crate::mermaid::render_diagram` (existing)
     - [x] `Heading` → heading HTML with slug/anchor (existing logic)
   - [x] Passthrough: `Prose` → identity (HTML already produced by Parse)

   **Caller updates:**
   - [x] Update `main.rs` callers: replace `render::markdown_to_html(&item.body)` with `render::render_blocks(&item.blocks)`
   - [x] Determine fate of `start_tag_to_html` / `end_tag_to_html` — likely dead after migration. Remove if unused.

   **Cruft + verification:**
   - [x] **Cruft audit:** Remove `markdown_to_html` (replaced by `render_blocks`), remove orphaned helper functions
   - [x] Preserve all existing render tests, adapt to new API

4. **Phase 4: Pipeline Clarity & Error Model** — Clean module boundaries, functor failure modes, and type-level phase separation

   **Error split:**
   - [x] Split `Error` enum into `ParseError` and `CompileError`:
     - [x] `ParseError::ReadFile`
     - [x] `ParseError::Frontmatter` (was `InvalidFrontmatter`)
     - [x] `ParseError::ContentDirNotFound`
     - [x] `ParseError::BrokenLink`
     - [x] `ParseError::Config`
     - [x] `CompileError::WriteFile`
     - [x] `CompileError::CreateDir`
     - [x] `CompileError::TemplateLoad`
     - [x] `CompileError::TemplateRender`
     - [x] `CompileError::CssBundle`
   - [x] Implement `From<ParseError> for Error` and `From<CompileError> for Error` for top-level `run()` boundary
   - [x] Update `content.rs` functions to return `ParseResult<T>`, `template_engine.rs` and `config.rs` functions to return `CompileResult<T>` / `ParseResult<T>`

   **Out-of-model code integration:**
   - [x] `generate_aliases` / `write_aliases`: consume `Content.frontmatter.aliases` (typed input from C), return `CompileResult<()>`
   - [x] `copy_static_assets` / `walk_dir`: kept at `Result<()>` — `walk_dir_inner` uses `ParseError::ReadFile` for directory listing (cross-phase)
   - [x] `bundle_css`: refactored from `Result<String, String>` to `CompileResult<String>` with `CompileError::CssBundle`
   - [x] `generate_feed` / `generate_sitemap_file`: narrowed to `CompileResult<()>` — underlying `generate_atom_feed`/`generate_sitemap` return `String` (infallible)

   **Cruft resolution (C2, C4-parameter, C5, C6):**
   - [x] Replace `process_pages()` with iteration over `manifest.pages` — **resolves C2**
   - [x] Remove `content_dir` parameter from `write_output()` — use `content.output_path` field — **resolves C5**
   - [x] Remove `content_dir` parameter from `ContentContext::from_content()` — use `content.output_path` field — **resolves C6**
   - [x] Remove `content_dir` parameter from `collect_tags()`, `generate_aliases()`, `generate_feed()`, `generate_sitemap_file()` where only used for `output_path` — **completes C4**

   **Hardcoded assumptions (H3-H6, H12-H17):**
   - [x] Evaluate template name hardcoding (H3-H6): extract `"page.html"`, `"content/default.html"`, `"section/default.html"`, `"tags/default.html"` to named constants or make configurable via `site.toml`. At minimum extract to module-level constants.
   - [x] Move `"Tagged: {}"` format string (H12) to template or `site.toml` config — display text should not be in Rust code
   - [x] Extract output filename constants: `"feed.xml"` (H13), `"sitemap.xml"` (H14), `"404.html"` (H15), `"index.html"` (H16) — at minimum named constants
   - [x] Unify template override pattern (H17): `render_page` should support `frontmatter.template.unwrap_or("page.html")` like `render_content` does — consistent behavior for a generic compiler

   **Pipeline clarity:**
   - [x] Audit `main.rs::run()` to ensure Parse completes fully before any Compile begins
   - [x] Ensure `collect_tags` uses `section.items` (stored field) instead of re-calling `collect_items()` _(already resolved in Phase 1 C4)_
   - [x] Add module-level doc comments to `content.rs` ("Parse functor: S → C") and `render.rs` ("Compile functor: C → O, Render sub-functor")
   - [x] Add **Type Mapping table** to `docs/models/sukr-compiler.md` Implementation Guidance section
   - [x] Evaluate `ContentKind` type split — defer: no friction from runtime kind checks observed. `Content.kind` field removed (dead). `ContentKind` enum retained as construction parameter for `output_path` branching.

   **Final cruft + verification:**
   - [ ] **Cruft audit (global):** Final pass to remove any dead types, orphaned functions, unused imports, holdover APIs. Pre-1.0 = zero tolerance for backwards-compat cruft.
   - [ ] Tests: functor composition, error phase separation, out-of-model operations return `CompileError` variants

## Verification

- [ ] `cargo test` passes after each phase
- [ ] `cargo clippy -- -D warnings` clean after each phase
- [ ] `cargo build` succeeds (no compile errors)
- [ ] Manual: build the actual site (`cargo run`) and verify HTML output is unchanged
- [ ] Broken link detection test: create content with a `[link](../nonexistent.md)` and verify compiler error
- [ ] Tag type safety test: `Tag("rust")` round-trips through serialization/deserialization
- [ ] ContentBlock catamorphism test: each variant renders to expected HTML in isolation
- [ ] Reference: compare `public/` output before and after refactor via `diff -r` to verify no regression

## Technical Debt

<!-- Populated during execution -->

| Item                                                                    | Severity | Why Introduced                                                                                                                   | Follow-Up                                  | Resolved |
| :---------------------------------------------------------------------- | :------- | :------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------- | :------: |
| `NavItem::PartialEq` ignores `path` and `children`                      | LOW      | Intentional for sort ordering in `BTreeSet` — equality based on `(weight, label)` discriminants only                             | Add doc comment on the impl                |          |
| ~~Unused `content_dir`/`content_root` params in 5 functions~~           | ~~LOW~~  | ~~`output_path` is now a field, but removing the params is a multi-file signature change~~                                       | ~~Phase 4 (C4 completion)~~                |   C11    |
| Magic literal `99` in Projects sort branch                              | LOW      | `DEFAULT_WEIGHT_HIGH` removed; value inlined pending sort logic migration                                                        | Phase 2 (SortKey adoption)                 |          |
| ~~`ContentBlock` variants never constructed~~                           | ~~LOW~~  | ~~Category C types defined in Commit 1; construction deferred to Phase 2 parse functor~~                                         | ~~Phase 2~~                                |  C5, C9  |
| ~~`SortKey::DateDesc`/`WeightTitle` never constructed~~                 | ~~LOW~~  | ~~SortKey enum defined in Commit 1; construction deferred to Phase 2 when sort-by-construction uses them~~                       | ~~Phase 2~~                                |   C12    |
| ~~`SortKey::for_content` never used (non-test)~~                        | ~~LOW~~  | ~~Constructor defined in Commit 1; sort logic was inlined into `discover_sections` in Commit 4~~                                 | ~~Phase 2 or remove if unused~~            |   C12    |
| ~~`Tag::new`/`as_str` never used (non-test)~~                           | ~~LOW~~  | ~~API defined in Commit 1; `Display` trait is what consumers use; `new`/`as_str` used only in tests~~                            | ~~Phase 2 or remove if unused~~            |   C12    |
| ~~`Content.kind` never read~~                                           | ~~LOW~~  | ~~Field added in Commit 2 for Category C; `blocks` consumed by `render_blocks` (C10); `kind` still unused~~                      | ~~Phase 4b (ContentKind split)~~           |   C15    |
| ~~`Event::Code` mapped to `Text` — loses inline code semantic~~         | ~~LOW~~  | ~~Resolved: inline code now renders as `<code>` in Prose blocks (C9), no separate variant needed~~                               | ~~N/A~~                                    |    C9    |
| `LinkTarget.source_line` always `None`                                  | LOW      | `Parser::new_ext` doesn't provide offsets; would need `into_offset_iter()`                                                       | Phase 2 reference validation               |          |
| ~~Duplicated `Options` flags in `parse_blocks` and `markdown_to_html`~~ | ~~LOW~~  | ~~Resolved: `markdown_to_html` removed (C10), only `parse_blocks` uses Options now~~                                             | ~~N/A~~                                    |   C10    |
| `SortKey` variants suppressed with `#[allow(dead_code)]`                | LOW      | Variants used by `Ord` impl but only constructed in `#[cfg(test)]` via `for_content`; production uses inline key construction    | Adopt `for_content` in `discover_sections` |          |
| `TAG_PAGE_TITLE_PREFIX` still in Rust code                              | LOW      | Extracted to constant (C13) but display text ideally belongs in template, not compiled code                                      | Move tag page title into Tera template     |          |
| Planned error variants not yet implemented                              | LOW      | Plan specified MissingSectionIndex, OrphanedNavEntry, RenderFailure, StaticAssetCopy — these error modes don't exist in code yet | Add when error conditions are implemented  |          |
| `copy_static_assets`/`walk_dir` return `Result` not `CompileResult`     | LOW      | `walk_dir_inner` uses `ParseError::ReadFile` for directory listing in compile-phase context — cross-phase dependency             | Introduce `CompileError::ReadDir` variant  |          |

## Deviation Log

<!-- Populated during execution -->

| Commit | Planned                                                                                                   | Actual                                                   | Rationale                                                                                                                                                                                                        |
| :----- | :-------------------------------------------------------------------------------------------------------- | :------------------------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C4     | `Section.items: BTreeMap<SortKey, Content>`                                                               | `Section.items: Vec<Content>` sorted at construction     | Vec is simpler and sufficient — items are immutable after construction, BTreeMap adds overhead without benefit                                                                                                   |
| C4     | — (not planned)                                                                                           | Removed `Section.path` and `Section.content_root`        | Discovered as vestigial after `collect_items()` removal — 0 external readers remained                                                                                                                            |
| C7     | `SiteManifest.nav: BTreeSet<NavItem>`                                                                     | `SiteManifest.nav: Vec<NavItem>` sorted at construction  | NavItem's PartialEq ignores path/children — BTreeSet would silently deduplicate items sharing (weight, label)                                                                                                    |
| P3     | Render catamorphism dispatches all 7 variants                                                             | 5-variant coproduct: Code, Math, Diagram, Heading, Prose | Model refined: Text, Link, Image removed. Text subsumed by Prose. Link/Image were overengineered — reference extraction is a Parse side-channel (`Content.links`), not a block type. Prose is the identity case. |
| C16    | Error split includes new variants (MissingSectionIndex, OrphanedNavEntry, RenderFailure, StaticAssetCopy) | Split existing 10 variants only — no new variants added  | New variants are aspirational: they represent error modes that currently don't exist in code. Adding empty variants would be dead code. Deferred to when the error conditions are actually implemented.          |

## Retrospective

<!-- Filled after execution -->

### Process

- Did the plan hold up? Where did we diverge and why?
- Were the estimates/appetite realistic?
- Did CHALLENGE catch the risks that actually materialized?

### Outcomes

- What unexpected debt was introduced?
- What would we do differently next cycle?

### Pipeline Improvements

- Should any axiom/persona/workflow be updated based on this experience?

## References

- Model: [`docs/models/sukr-compiler.md`](file:///var/home/nrd/git/github.com/nrdxp/nrd.sh/docs/models/sukr-compiler.md)
- Sketch: [`.sketches/2026-02-21-sukr-formal-model.md`](file:///var/home/nrd/git/github.com/nrdxp/nrd.sh/.sketches/2026-02-21-sukr-formal-model.md)
