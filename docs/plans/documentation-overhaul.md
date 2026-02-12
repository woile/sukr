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

- [ ] `cargo test` passes (no source code changes, but hygiene check)
- [ ] Docs site builds: `cd docs && sukr` (assumes `sukr` in PATH via `cargo install --path .`)
- [ ] `grep -ri "Sukr" docs/ README.md` returns zero results (only lowercase "sukr")
- [ ] Built docs site: homepage no longer claims `cargo install sukr`
- [ ] Built docs site: getting-started tutorial has working templates and completion step
- [ ] Built docs site: deployment page exists and renders
- [ ] Built docs site: no broken internal links

## Technical Debt

| Item                                                                                                                                                            | Severity | Why Introduced                                          | Follow-Up                                                        | Resolved |
| :-------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------- | :------------------------------------------------------ | :--------------------------------------------------------------- | :------: |
| `deployment.md` and `content-organization.md` both have weight 1 — alphabetical sort puts Content Org before Deployment in sidebar, suboptimal for user journey | Low      | Changing other page weights was scope creep for Phase 3 | Adjust weights to match Install → Deploy → Organize user journey |    ☑    |

## Retrospective

_To be filled after execution._

## References

- Sketch: `.sketches/2026-02-11-documentation-improvements.md`
