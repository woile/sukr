# PLAN: sukr 1.0 API Stabilization

<!--
  Source sketch: .sketches/2026-02-13-api-stabilization.md
  Selected approach: B+ (essential features + cleanup + aliases + date validation)
  Key pivot: YAML frontmatter → TOML frontmatter (CHALLENGE finding)
-->

## Goal

Implement the pre-1.0 API changes required to stabilize sukr's public contract: switch frontmatter from hand-rolled YAML to serde-backed TOML, normalize template variable naming, add missing features (draft mode, 404 page, tag listing pages, aliases, date validation, feed/sitemap config), remove dead code, and add template fallback behavior. After this work, the five public surfaces (site.toml schema, frontmatter fields, template variables, CLI, content directory conventions) are locked — post-1.0 breaking changes require explicit user approval.

## Constraints

- Pre-1.0: breaking changes are acceptable now but the goal is to make them unnecessary after this work
- Suckless philosophy: no speculative features, no new dependencies unless already transitively present
- `tree-house` (git dep) and `lightningcss` (alpha) are accepted risks — do not attempt to resolve
- `chrono` is acceptable for date validation — it's already a transitive dependency via `tera`
- Every surface stabilized is a surface committed to maintaining

## Decisions

| Decision               | Choice                                                                                             | Rationale                                                                                                                                                 |
| :--------------------- | :------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontmatter format     | **TOML** (replacing hand-rolled YAML)                                                              | `toml` crate + serde already in deps. Eliminates the fragile hand-rolled parser. Every future field is just a struct field with `#[derive(Deserialize)]`. |
| Frontmatter delimiter  | `+++`                                                                                              | Hugo convention for TOML frontmatter. Unambiguous — no risk of confusion with YAML `---` or Markdown horizontal rules.                                    |
| Template naming        | Mirror site.toml structure (`config.nav.nested` not `config.nested_nav`)                           | Consistency between config and templates; pre-1.0 is only window for this break                                                                           |
| Date type              | `Option<chrono::NaiveDate>` with custom `deserialize_date` fn for TOML native dates                | Parse, don't validate. Custom serde deserializer accepts `toml::Datetime`, extracts date, constructs `NaiveDate`. Invalid dates fail at deser.            |
| Draft filtering        | `draft: bool` (`#[serde(default)]`) in `Frontmatter`, filter in `collect_items()` and `discover()` | Filter early so drafts don't appear in nav, listings, sitemap, or feed.                                                                                   |
| Feed/sitemap config    | `[feed]` and `[sitemap]` tables with `enabled` boolean in `SiteConfig`                             | Users need opt-out. Default `true` preserves backward compat.                                                                                             |
| Tag listing pages      | Generate `/tags/<tag>.html` using a new `tags/default.html` template                               | Minimal approach — one template, one generation loop. No pagination.                                                                                      |
| Aliases                | `aliases = ["/old/path"]` in frontmatter, generate HTML redirect stubs                             | Standard pattern (Hugo). `<meta http-equiv="refresh">` redirect.                                                                                          |
| 404 page               | `content/404.md` → `404.html` at output root                                                       | Simplest approach. Most static hosts auto-serve `/404.html`.                                                                                              |
| Template fallback      | Try `section/<type>.html`, fall back to `section/default.html`                                     | Removes the requirement to create a template for every section_type.                                                                                      |
| Dead template cleanup  | Delete `section/features.html` and `homepage.html`                                                 | Byte-for-byte duplicate and dead code respectively.                                                                                                       |
| `base_url` duplication | Remove top-level `base_url` template variable                                                      | Single source of truth via `config.base_url`.                                                                                                             |
| Tags syntax            | `tags = ["foo", "bar"]` (flat TOML array)                                                          | Replaces nested `taxonomies.tags` YAML. Simpler, no indirection.                                                                                          |

## Risks & Assumptions

| Risk / Assumption                                       | Severity | Status      | Mitigation / Evidence                                                                                                                                                                                                |
| :------------------------------------------------------ | :------- | :---------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| TOML frontmatter breaks all 17 existing content files   | **HIGH** | Validated   | One-time mechanical migration. All files are simple key-value; no complex structures. Migration is part of Phase 1.                                                                                                  |
| Documentation pages embed YAML frontmatter examples     | MEDIUM   | Validated   | `configuration.md`, `content-organization.md`, `getting-started.md`, `feeds.md`, `sections.md`, `sitemap.md`, `templates.md` all contain example frontmatter in their body text. Must update these doc examples too. |
| Template naming break affects existing user templates   | MEDIUM   | Validated   | Only `docs/templates/base.html` references `config.nested_nav` and `base_url`. The docs site is the only known deployment.                                                                                           |
| Tag listing pages need a template but no default ships  | MEDIUM   | Unvalidated | Must design and ship `tags/default.html` in `docs/templates/`.                                                                                                                                                       |
| `collect_items()` triple-call not fixed by this plan    | LOW      | Accepted    | Performance issue, not API concern. Deferred.                                                                                                                                                                        |
| Removing `base_url` top-level variable breaks templates | MEDIUM   | Validated   | Only `docs/templates/base.html` uses it.                                                                                                                                                                             |

## Open Questions

All resolved during CHALLENGE. See CHALLENGE Notes below.

### CHALLENGE Notes

Items presented to nrd for decision:

1. **YAML → TOML frontmatter switch.** The hand-rolled YAML parser can't handle `aliases` lists. Rather than extending a fragile parser, nrd decided to switch frontmatter to TOML — the `toml` crate and serde are already dependencies. This eliminates the parser entirely and replaces 70 lines of hand-rolled code with `#[derive(Deserialize)]`. **Decision: switch to TOML.**

Items validated by codebase investigation:

2. **`ConfigContext` normalization is clean.** Flat struct at `template_engine.rs:139-155`. Refactoring to nested `nav: NavContext { nested, toc }` is straightforward.
3. **`base_url` duplication confirmed.** `template_engine.rs:118` injects standalone `base_url`. `ConfigContext.base_url` at line 142 is canonical. Removal safe.
4. **Template variable `content` is correct.** Lines 57 and 83 inject rendered HTML as `content`, matching the sketch contract.
5. **Phase ordering is sound.** No circular dependencies between phases.
6. **17 content files need migration.** All simple frontmatter — no nested structures beyond `taxonomies.tags` (which becomes flat `tags = [...]`). 7 files also contain embedded YAML examples in body text that need updating.

## Scope

### In Scope

1. **TOML frontmatter switch** — replace hand-rolled YAML parser with `#[derive(Deserialize)]` + `toml::from_str`, migrate 17 content files, change delimiter from `---` to `+++`
2. Template naming normalization (`config.nested_nav` → `config.nav.nested`, add `config.nav.toc`, remove duplicate `base_url`)
3. `draft` frontmatter field + filtering
4. `aliases` frontmatter field + redirect stub generation
5. Date validation (YYYY-MM-DD) at parse time
6. `[feed].enabled` and `[sitemap].enabled` config
7. `content/404.md` → `404.html` support
8. Tag listing page generation (`/tags/<tag>.html`)
9. Template section fallback (`section/<type>.html` → `section/default.html`)
10. Dead template removal (`section/features.html`, `homepage.html`)
11. Tests for all new and changed behavior

### Out of Scope

- Pagination
- Asset co-location
- i18n
- Verbose/quiet CLI flags
- `collect_items()` caching
- `ContentKind` refactoring
- Magic string enum extraction

## Phases

1. **Phase 1: TOML Frontmatter & Config Normalization** — replace the parser, migrate content, fix naming
   - [x] Replace `Frontmatter` struct with `#[derive(Deserialize)]`
   - [x] Add new fields: `draft: bool` (`#[serde(default)]`), `aliases: Vec<String>` (`#[serde(default)]`), keep all existing fields
   - [x] Replace `parse_frontmatter()` with `toml::from_str::<Frontmatter>()`
   - [x] Update `extract_frontmatter()` to detect `+++` delimiters instead of `---`
   - [x] Add date validation: custom `deserialize_date` fn for TOML native dates → `chrono::NaiveDate`
   - [x] Change `tags` from `taxonomies.tags` nesting to flat `tags = ["..."]` (direct TOML array)
   - [x] Migrate all 17 content files from YAML (`---`) to TOML (`+++`) frontmatter
   - [x] Update embedded frontmatter examples in documentation pages (7 files)
   - [x] Add `FeedConfig` and `SitemapConfig` structs to `config.rs` with `enabled: bool` (default `true`)
   - [x] Wire feed/sitemap config into `SiteConfig` deserialization
   - [ ] Gate feed generation in `main.rs` on `config.feed.enabled`
   - [ ] Gate sitemap generation in `main.rs` on `config.sitemap.enabled`
   - [x] Refactor `ConfigContext`: flat `nested_nav: bool` → nested `nav: NavContext { nested, toc }`
   - [x] Remove duplicate `base_url` top-level template variable injection
   - [x] Update `docs/templates/base.html`: `config.nested_nav` → `config.nav.nested`, `base_url` → `config.base_url`
   - [x] Delete `docs/templates/section/features.html` and `docs/templates/homepage.html`
   - [x] Add template section fallback in `render_section`: try `section/<type>.html`, fall back to `section/default.html`
   - [x] Update/fix all existing tests to use TOML frontmatter
   - [ ] Add new tests: TOML parsing, date validation (valid + invalid), feed/sitemap config gating
   - [ ] Verify all 69 existing tests pass (updated for TOML)

2. **Phase 2: Draft & Alias Features** — implement filtering and redirect generation
   - [ ] Filter items where `draft == true` from `collect_items()` results
   - [ ] Filter drafts from `SiteManifest.posts` during discovery
   - [ ] Filter drafts from nav discovery (`discover_nav()`)
   - [ ] Filter drafts from sitemap entries
   - [ ] Filter drafts from feed entries
   - [ ] Generate HTML redirect stubs for each alias path (`<meta http-equiv="refresh">`)
   - [ ] Add tests: draft filtering (excluded from listing, nav, feed, sitemap)
   - [ ] Add tests: alias redirect generation (valid HTML, correct target URL)

3. **Phase 3: 404 & Tag Pages** — new content generation features
   - [ ] Detect `content/404.md` in content discovery, treat as special page
   - [ ] Render `404.md` to `404.html` in output root
   - [ ] Collect all unique tags across content items during build
   - [ ] Create `tags/default.html` template in `docs/templates/`
   - [ ] Generate `/tags/<tag>.html` for each unique tag with list of tagged items
   - [ ] Add tag listing page entries to sitemap (if enabled)
   - [ ] Add tests: 404 page generation
   - [ ] Add tests: tag listing page generation (correct paths, correct items per tag)
   - [ ] End-to-end: build `docs/` site and verify all outputs

## Verification

- [ ] `cargo test` — all existing tests pass (updated for TOML frontmatter)
- [ ] `cargo test` — all new tests pass (minimum 12 new tests across 3 phases)
- [ ] `cargo clippy -- -D warnings` — no warnings
- [ ] `cargo build` — clean compilation
- [ ] End-to-end: build `docs/` site with `cargo run`, verify:
  - [ ] `public/sitemap.xml` exists (default enabled)
  - [ ] `public/atom.xml` exists (default enabled)
  - [ ] `public/404.html` exists (with 404.md in docs/content)
  - [ ] Templates use `config.nav.nested` (not `config.nested_nav`)
  - [ ] Templates use `config.base_url` (not bare `base_url`)
  - [ ] No `section/features.html` or `homepage.html` templates remain

## Technical Debt

<!--
  Populated during execution. Empty at plan creation.
-->

| Item | Severity | Why Introduced | Follow-Up | Resolved |
| :--- | :------- | :------------- | :-------- | :------: |

## Retrospective

<!--
  Filled in after execution is complete.
-->

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

- Charter: `docs/charters/sukr-v1.md`
- Sketch: `.sketches/2026-02-13-api-stabilization.md`
