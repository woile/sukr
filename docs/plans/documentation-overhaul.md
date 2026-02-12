# PLAN: Documentation Overhaul

## Goal

Bring sukr's docs site into full axiom compliance. Fix stale content (6 items), resolve structural violations (8 pages), and fill content gaps — producing a docs site that serves the Evaluate → Install → Build → Deploy → Customize user journey without misleading, dead-ending, or duplicating information.

## Constraints

- Documentation axiom governs all output (quadrant discipline, audience declaration, scope economy)
- sukr builds its own docs site — every change is verifiable via `sukr` build
- Template authoring links to upstream Tera docs, not a custom tutorial
- Developer-facing READMEs (`queries/`) are out of scope for user-facing docs site changes
- `themes/README.md` is a targeted exception: trimmed because theming content moves to docs site

## Decisions

| Decision                  | Choice                                                              | Rationale                                                                                                                      |
| :------------------------ | :------------------------------------------------------------------ | :----------------------------------------------------------------------------------------------------------------------------- |
| Phase ordering            | Fix stale → restructure → fill gaps → cross-refs                    | Stale fixes are highest risk (users get stuck). Later phases depend on earlier pages being correct first.                      |
| sections.md consolidation | Slim to reference-only, move explanation to content-organization.md | Respects quadrant split: explanation lives in content-organization, reference lives in features/sections. Preserves nav entry. |
| Theming docs              | Add section to syntax-highlighting.md                               | Avoids creating a new theming.md that would drift from themes/README.md. Themes ARE syntax highlighting — same topic.          |
| Deployment guide approach | Generic "copy public/ to any static host" + platform links          | Suckless: no embedded platform configs to maintain.                                                                            |
| Tutorial templates        | Inline minimal template set in getting-started.md                   | Scope economy: a starter repo is a separate project to maintain. Three inline templates are sufficient.                        |
| CLI reference             | Defer                                                               | One flag (`-c`). configuration.md already covers it. Create when CLI surface grows.                                            |
| Error reference           | Out of scope                                                        | Error messages should be self-explanatory. If not, that's a code fix. This isn't API documentation.                            |
| Homepage rewrite          | Explanation-primary with tutorial link                              | Homepage ≠ standard document. Allow a concise "get started" link but don't embed a full quick-start.                           |

## Risks & Assumptions

| Risk / Assumption                               | Severity | Status    | Mitigation / Evidence                                                                                                |
| :---------------------------------------------- | :------- | :-------- | :------------------------------------------------------------------------------------------------------------------- |
| Deployment guide goes stale                     | MEDIUM   | Mitigated | Keep generic — "copy public/ to any host" + links. No platform-specific configs.                                     |
| Theming section creates sync with themes/README | MEDIUM   | Mitigated | Add to existing syntax-highlighting.md. Reference themes/README for available theme list only.                       |
| Section consolidation breaks nav discovery      | LOW      | Mitigated | Keep sections.md as slimmed reference page. Users browsing features/ still find it.                                  |
| Phases 2 and 3 not fully independent            | LOW      | Accepted  | Homepage gets a second touch in Phase 3 to add cross-links to new pages. Acknowledged, not a blocker.                |
| Quadrant discipline improves the docs           | —        | Validated | Worst audit pages (homepage, README) mix quadrants most. Best pages (comparison, math, mermaid) are single-quadrant. |
| Inline tutorial templates are sufficient        | —        | Validated | Docs site templates are tiny: base.html=67 lines, page.html=10 lines. Minimal tutorial set is ~30 lines total.       |

## Open Questions

None. All CHALLENGE questions resolved.

## Scope

### In Scope

- All committed, user-facing docs site content (`docs/content/`, `docs/templates/`, `docs/site.toml`)
- `README.md` (GitHub-facing — restructure for quadrant discipline)
- `themes/README.md` (targeted exception: trim since theming content moves to docs site)
- New content: `deployment.md` (how-to), theming section in `syntax-highlighting.md`

### Out of Scope

- `rearch.md` (untracked, not part of the codebase)
- `queries/README.md` (developer-facing)
- CLI reference page (deferred — one flag)
- Error reference page (error messages should be self-explanatory)
- Tera template tutorial (link upstream)
- Plugin/API documentation

## Phases

1. **Phase 1: Fix Stale Content** — every factual claim matches reality
   - [x] S1: Replace `cargo install sukr` in `_index.md` with `cargo install --path .` after clone
   - [x] S4: Replace Step 4 dead-end in `getting-started.md` with inline minimal templates (base.html, page.html, content/default.html)
   - [x] S5: Normalize "Sukr" → "sukr" in `architecture.md`
   - [x] S6: Normalize `title` in `docs/site.toml`
   - [x] S7: Fix template override path in `templates.md` (`page/special.html` → `content/special.html`)
   - [x] S9: Update copyright in `docs/templates/base.html` (2024 → 2026, or remove if unnecessary for OSS)
   - [x] Add "view your site" final step to `getting-started.md` with expected output

2. **Phase 2: Structural Rework** — every page declares one quadrant, serves one audience
   - [x] Rewrite `_index.md` as explanation quadrant (what sukr is, why it exists, link to tutorial)
   - [x] Slim `features/sections.md` to reference-only (section_type field, template dispatch)
   - [x] Move explanation content from `sections.md` into `content-organization.md`
   - [x] Add link to upstream Tera docs in `features/templates.md`
   - [x] Add defaults column to frontmatter table in `configuration.md`
   - [x] Add "Theming" section to `syntax-highlighting.md` (choosing/customizing themes)

3. **Phase 3: Fill Content Gaps** — cover Deploy and Customize stages of user journey
   - [x] Create `docs/content/deployment.md` (how-to: generic static host deployment + platform links)
   - [x] Update `_index.md` to cross-link to new deployment page
   - [x] Update `getting-started.md` "Next Steps" to include deployment

4. **Phase 4: README + Cross-references** — README serves as focused explanation, cross-links established
   - [x] Restructure `README.md` to explanation quadrant (what, why, how it compares)
   - [x] Remove duplicated comparison table (link to docs site `comparison.md`)
   - [x] Remove inline config reference (link to docs site `configuration.md`)
   - [x] Keep security overview (short, relevant for trust evaluation)
   - [x] Trim `themes/README.md` to attribution + structure + link to docs site (targeted exception)
   - [x] Add cross-references between docs pages where missing

## Verification

- [x] `cargo test` passes — 69/69 (no source code changes, hygiene check)
- [x] Docs site builds: `cargo run -- -c docs/site.toml` succeeds, all pages rendered
- [x] No uppercase `Sukr` in docs or README (all lowercase `sukr`)
- [x] No `cargo install sukr` in docs content
- [x] Getting-started tutorial has working templates (base.html, page.html, default.html) and completion step ("View your site")
- [x] Deployment page exists and renders at `docs/public/deployment.html`
- [x] No broken internal cross-reference links (all navigation links resolve; "broken" hits are example/illustrative paths in prose)

## Technical Debt

| Item                                                                                                                                                            | Severity | Why Introduced                                          | Follow-Up                                                        | Resolved |
| :-------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------- | :------------------------------------------------------ | :--------------------------------------------------------------- | :------: |
| `deployment.md` and `content-organization.md` both have weight 1 — alphabetical sort puts Content Org before Deployment in sidebar, suboptimal for user journey | Low      | Changing other page weights was scope creep for Phase 3 | Adjust weights to match Install → Deploy → Organize user journey |    ☑    |

## Retrospective

### What went well

**Phased approach payoff.** The 4-phase structure (surface → structural → gaps → README) prevented scope creep by giving each commit a clear boundary. Phases 1 and 2 could be verified independently before touching cross-references or creating new pages. Total execution was 7 commits across 4 phases — manageable, reviewable.

**Source verification during planning.** CHALLENGE phase verified claims against source code before committing to plan scope. Finding S7 (template override path `page/special.html` vs. source's `content/special.html`) and the frontmatter defaults (verified against `content.rs`) prevented shipping incorrect documentation. The CHALLENGE also correctly identified 3 false positives (S2, S3, S8) that would have been wasted effort.

**CHALLENGE phase earned its keep.** Beyond false positive elimination, CHALLENGE produced two durable architectural decisions: (1) theming content goes into `syntax-highlighting.md` rather than a standalone `theming.md` page (avoids sync drift with `themes/README.md`), and (2) deployment guide uses platform links rather than embedded platform-specific config snippets (avoids maintenance burden). Both decisions were validated by nrd and shaped better outcomes than the initial PROPOSE.

**Sections consolidation.** The option (b) approach — slim `sections.md` to reference-only, move explanation into `content-organization.md` — preserved both navigation entries while eliminating content duplication. Clean quadrant discipline: explanation stays in one place, reference in another.

### What surprised us

**Sketch accuracy:** 3 of 9 surface findings were false positives (correct repo URL, not wrong). The initial audit assumed `nrdxp/sukr` was wrong because the repo is `nrdxp/nrd.sh`, but `nrdxp/sukr` is the correct canonical URL for cloning. Lesson: always verify before planning, not just before executing.

**Extra Sukr instances:** Plan flagged `architecture.md` L8 for uppercase "Sukr," but execution found 3 additional instances (L71, L82, L124). Minor divergence — same fix applied consistently. Reinforces the value of `grep` over line-targeted assumptions.

**Tera link existed:** Sketch's §11 finding said `templates.md` had "no link to upstream Tera docs." During execution, the link was already present — just at a stale URL (`tera.netlify.app`). Step adapted: update URL + add authoring syntax pointer, rather than add a new link from scratch. Sketches capture impressions; execution verifies them.

### Process observations

**CORE 2-commit granularity was right-sized.** Each phase fit cleanly into 1–2 commits. Phase 1's split (tutorial fix vs. mechanical fixes) kept diffs reviewable. Phase 2's split (structural rework vs. enrichments) separated destructive changes (rewriting, consolidating) from additive ones (new columns, new sections). The maximum of 2–3 commits per CORE kept scope bounded.

**Cross-phase dependencies are minor but real.** Phase 3 (deployment.md) touched Phase 2's output (`_index.md`'s Learn More section). Phase 4 (weight fix) resolved Phase 3's tech debt. These weren't blockers — just small edits to existing files — but they show that purely independent phases are a useful fiction for planning, not execution reality.

**Tech debt tracking worked.** The nav weight collision surfaced during Phase 3 execution, was recorded in the plan's Technical Debt table, and resolved in Phase 4 step 4. The table mechanic (Item / Severity / Why / Follow-up / Resolved) provided clean traceability from discovery to resolution.

### Intentional deferrals

| Item                                | Rationale                                                                                                                                                        |
| :---------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| G2: Troubleshooting/error reference | sukr errors are self-descriptive. Unclear errors are a code fix, not a docs page. No user signal that current messages cause confusion.                          |
| G5: Dedicated CLI reference page    | One flag (`-c`). CLI section added to `configuration.md` instead. Will revisit when CLI surface grows.                                                           |
| `security.md` mixed quadrants       | Flagged in sketch (trust model = reference, CSP headers = how-to). Left as-is — page is short, splitting would create two thin pages. Pragmatic exception to §7. |

## References

- Sketch: `.sketches/2026-02-11-documentation-improvements.md`
