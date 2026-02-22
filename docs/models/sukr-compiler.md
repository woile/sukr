# MODEL: Sukr Compiler Architecture

<!--
  MODEL document produced by the /model workflow (Create mode).
  Formalizes the essential structure of the Sukr static site compiler
  as three categories connected by two functors.

  Sketch: .sketches/2026-02-21-sukr-formal-model.md
  See: workflows/model.md for the full protocol specification.
  See: personas/sdma.md for the applied modeling toolkit.
  See: axioms/formal-foundations.md for the mathematical foundations.
-->

## Domain Classification

**Problem Statement:**

Sukr is a suckless static site compiler. It deterministically transforms a
content directory into a zero-JS static website. This model formalizes Sukr's
structure from first principles, providing a measuring stick for architectural
completeness independent of the current implementation. If the formalism
implies rearchitecture, that is a useful finding.

**Domain Characteristics:**

- **Finite, inductively defined input:** Markdown files with TOML frontmatter
  form a tree (site → sections → pages → content blocks). All input is known at
  build time.
- **Scope Boundary:** This model covers the _content transformation pipeline_.
  Verbatim file operations (e.g., static asset copying) are out of scope: they
  involve no content model participation, no cross-dependencies, and no non-trivial
  error modes, so they lack the structure that warrants categorical representation.
- **Deterministic transformation:** The output is a pure function of the input:
  `(ContentDir, Config) → OutputDir`. No runtime state, no user interaction.
- **Cross-cutting dependencies:** Pages are not fully independent. Navigation
  trees, tag taxonomies, feeds, and inter-page links create structural
  relationships that must be validated at compile time.
- **Algebraic identity:** Content is defined by its _construction_ (frontmatter
  fields, block types, section hierarchy), not by _observation_. This places the
  domain firmly on the algebra side of the algebra/coalgebra duality
  (formal-foundations §4).

## Formalism Selection

| Aspect                  | Detail                                                                                                                                                                                                                                                                                                                      |
| :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Primary Formalism**   | Categories and Functors (algebraic)                                                                                                                                                                                                                                                                                         |
| **Supporting Tools**    | Olog labeling conventions (SDMA §2), catamorphisms with coproduct dispatch                                                                                                                                                                                                                                                  |
| **Decision Matrix Row** | None. Sukr's domain is below the SDMA's target complexity. The SDMA toolkit provides the foundational vocabulary; the specialized rows (coalgebra, session types, LinRel, etc.) do not apply.                                                                                                                               |
| **Rationale**           | Three small categories connected by two functors is the minimal representation that faithfully captures both the content model (types and relationships) and the transformation pipeline (phases and module boundaries), while making cross-dependency validation and error handling structural (functor well-definedness). |

**Alternatives Considered:**

- **SMC via String Diagrams + Coalgebras** (Gemini's original proposal):
  Rejected. SMC requires a tensor product (parallel composition), braiding,
  associator, and unit object, none of which have domain counterparts. Sukr's
  parallelism is embarrassingly parallel fork-join, not structural monoidal
  composition. Coalgebras model systems with hidden state and potentially
  infinite behavior; Sukr has neither. See sketch for exhaustive analysis.

- **Olog + Catamorphism (Pure Algebraic):** Not rejected, but _subsumed_.
  The olog IS the Content category. The catamorphism IS the internal structure
  of the Compile functor. Choosing functors doesn't add machinery over ologs; it
  provides the connecting tissue for cross-phase validation.

- **Linear Type Theory (MILL):** Rejected. Sukr has no non-duplicable resources,
  no ownership transfer semantics, and no linearity constraints. Files are
  freely readable and outputs are generated once.

## Model

### Overview: Three Categories, Two Functors

```
Source ──Parse──→ Content ──Compile──→ Output
```

Three small categories connected by two functors. **Content** is the central
category; it specifies what constitutes a valid site. The two functors define
the compiler's two phases.

---

### Category S: Source

The Source category models the filesystem inputs and configuration.

**Objects:**

| Object         | Definition                                  |
| :------------- | :------------------------------------------ |
| A file         | A filesystem entry with a path and contents |
| A directory    | A filesystem entry containing other entries |
| A config entry | A key-value pair from `site.toml`           |
| A config       | The complete `site.toml` configuration      |

**Morphisms:**

| Morphism  | Type                      | Semantics                |
| :-------- | :------------------------ | :----------------------- |
| contains  | A directory → A file      | Directory containment    |
| contains  | A directory → A directory | Directory nesting        |
| specifies | A config → A config entry | Configuration membership |
| root      | A config → A directory    | The content root path    |

**Commutative Diagrams (Invariants):**

- **Config path resolution:** A config →[root]→ A directory must refer to a
  directory that exists as an object in S. (The content directory must exist.)

---

### Category C: Content

The Content category is the domain model. It follows olog labeling conventions
(SDMA §2: singular indefinite noun phrases, functional morphisms).

**Objects:**

| Object            | Definition                                       |
| :---------------- | :----------------------------------------------- |
| A site manifest   | The complete content structure of a site         |
| A section         | A directory of related pages with an `_index.md` |
| A page            | A single content file with frontmatter and body  |
| A homepage        | The root `_index.md` (a distinguished page)      |
| A custom 404 page | The `_404.md` file (a distinguished page)        |
| A frontmatter     | The TOML metadata header of a page               |
| A content block   | An atomic unit of page body content              |
| A nav item        | An entry in the navigation tree                  |
| A tag             | A classification label for pages                 |

**Morphisms (Aspects):**

Each morphism is functional: every element of the domain maps to exactly one
element of the codomain.

| Morphism        | Type                           | Semantics                                              |
| :-------------- | :----------------------------- | :----------------------------------------------------- |
| has frontmatter | A page → A frontmatter         | Each page has exactly one frontmatter                  |
| belongs to      | A page → A section             | Each non-root page belongs to one section              |
| has index       | A section → A page             | Each section has exactly one index page                |
| has weight      | A page → ℤ                     | Sort order (default: 0)                                |
| has title       | A frontmatter → String         | The page title                                         |
| has slug        | A page → String                | URL-safe identifier (derived from path or frontmatter) |
| has body        | A page → [A content block]     | Ordered sequence of content blocks                     |
| references      | A page → A page                | Inter-page link (partial: only if links exist)         |
| tagged with     | A page → A tag                 | Tag association (partial: only if tags exist)          |
| nav child       | A nav item → A nav item        | Navigation tree nesting                                |
| contains        | A site manifest → A section    | Section membership                                     |
| contains        | A site manifest → A page       | Root-level page membership                             |
| has nav         | A site manifest → [A nav item] | The navigation tree                                    |
| has homepage    | A site manifest → A homepage   | The root index page                                    |

**Content Block Algebra (Coproduct):**

A content block is an algebraic data type (a tagged union with per-variant
structure):

```
ContentBlock = Code(Language, String)      -- tree-sitter syntax highlighting
             | Math(String, DisplayMode)   -- KaTeX → MathML
             | Diagram(String)             -- Mermaid → SVG
             | Heading(Level, String, Id)  -- slug generation, anchor extraction
             | Prose(HTML)                 -- standard rendering (identity)
```

The first four variants are interception points: domain-specific transformations
that a standard markdown renderer cannot perform. `Prose` is the identity
case — standard markdown rendering where the parser library's output is the
desired output. Every variant earns its existence; nothing is modeled that
sukr does not transform or that does not require distinct treatment.

Reference extraction (the `references` morphism) is a side-channel of parsing,
not a property of the block algebra. Internal link URLs are extracted directly
from the markdown event stream and stored on the page, independent of
ContentBlock construction.

**Commutative Diagrams (Invariants):**

1. **Output path determination:** The composition
   `A page →[belongs to]→ A section →[has slug]→ String` combined with
   `A page →[has slug]→ String` determines the output path. This diagram must
   commute: the output path is uniquely determined.

2. **Nav from structure:** The navigation tree is derivable from the section
   and page structure plus weights. `A site manifest →[has nav]→ [A nav item]`
   must be consistent with `A site manifest →[contains]→ A section →[has index]→ A page →[has weight]→ ℤ`.

3. **Reference integrity:** For every morphism `A page →[references]→ A page`,
   the target must exist as an object in C. This is not separately enforced;
   it follows from C being a well-formed category (morphisms only exist
   between existing objects).

4. **Tag closure:** For every morphism `A page →[tagged with]→ A tag`, the tag
   must exist as an object in C.

---

### Category O: Output

The Output category models the generated site.

**Objects:**

| Object              | Definition                       |
| :------------------ | :------------------------------- |
| An HTML file        | A generated page file            |
| A feed file         | The Atom XML feed                |
| A sitemap file      | The `sitemap.xml`                |
| A tag page          | A generated index page for a tag |
| An output directory | A directory in the output tree   |
| A redirect file     | An HTML redirect for URL aliases |

**Morphisms:**

| Morphism   | Type                               | Semantics                       |
| :--------- | :--------------------------------- | :------------------------------ |
| resides in | An HTML file → An output directory | File placement                  |
| links to   | An HTML file → An HTML file        | Hyperlinks between output pages |

**Key property:** O is structurally simpler than C. Much of C's relational
structure (sections, weights, tags) is _consumed_ by the Compile functor and
does not appear in O. It gets baked into output file content and directory
structure.

---

### Functor Parse: S → C

The Parse functor discovers content structure from the filesystem.

**Action on objects:**

| Source object                       | Maps to              | Notes              |
| :---------------------------------- | :------------------- | :----------------- |
| A directory with `_index.md`        | A section            | Section discovery  |
| A `.md` file with valid frontmatter | A page               | Page discovery     |
| Root `_index.md`                    | A homepage           | Distinguished page |
| `_404.md`                           | A custom 404 page    | Distinguished page |
| `site.toml` entries                 | Frontmatter defaults | Config integration |

**Action on morphisms:**

- Directory containment in S maps to the `belongs to` morphism in C
- File contents map to the `has body` morphism (markdown → content blocks)
- Internal links in markdown map to `references` morphisms in C (extracted as
  a side-channel during content block parsing)
- Tag fields in frontmatter map to `tagged with` morphisms in C

**Functor laws:**

- **Identity preservation:** A file that maps to itself (identity morphism in
  S) must map to an identity morphism in C.
- **Composition preservation:** If `file ⊂ dir ⊂ root`, then
  `Parse(file ⊂ root) = Parse(file ⊂ dir) ∘ Parse(dir ⊂ root)`. Content
  hierarchy must reflect filesystem hierarchy.

**Parse errors (functor failure):**

Parse is a partial functor, undefined on invalid inputs. Failure modes:

| Failure                                                | Formal meaning                                        | Error                                      |
| :----------------------------------------------------- | :---------------------------------------------------- | :----------------------------------------- |
| Missing `_index.md` in a directory with `.md` children | Directory containment morphism in S has no image in C | "Section directory `X` has no `_index.md`" |
| Invalid TOML frontmatter                               | File object in S has no valid Page image in C         | "Failed to parse frontmatter in `X`"       |
| Missing `---` delimiters                               | File is not a valid content file                      | "No frontmatter found in `X`"              |
| Non-UTF-8 content                                      | File object has no string representation              | "File `X` is not valid UTF-8"              |

---

### Functor Compile: C → O

The Compile functor transforms the content model into the output site.

**Action on objects:**

| Content object                       | Maps to                          | Notes                |
| :----------------------------------- | :------------------------------- | :------------------- |
| A page                               | An HTML file                     | Page rendering       |
| A section                            | An output directory + index HTML | Section rendering    |
| A homepage                           | Root `index.html`                | Homepage rendering   |
| A tag                                | A tag page                       | Tag index generation |
| A site manifest (if feed enabled)    | A feed file                      | Atom feed            |
| A site manifest (if sitemap enabled) | A sitemap file                   | Sitemap XML          |

**Action on morphisms:**

- `references` in C maps to `links to` in O (cross-dependency validation)
- `belongs to` in C maps to `resides in` in O (directory structure)
- `tagged with` in C maps to hyperlinks from tag pages to page files

**Internal decomposition** (implementation guidance, not separate categories):

```
Compile = Emit ∘ Template ∘ Render
```

Where:

- **Render:** Catamorphism over the content block algebra. Fold over `[ContentBlock]`
  with coproduct dispatch:

  ```
  render : ContentBlock → HTML
  render(Code(lang, src))      = highlight(lang, src)
  render(Math(tex, mode))      = katex(tex, mode)
  render(Diagram(src))         = mermaid_svg(src)
  render(Heading(level, text)) = heading_html(level, text)
  render(Prose(html))          = html
  ```

  The first four cases are the interception points that justify sukr's
  existence. `Prose` is the identity morphism — content the parser library
  already rendered correctly.

- **Template:** Applies the page template with navigation context, metadata,
  and site-wide configuration. Note that templates are a _parameter_ defining
  the Compile functor, not a separate input category. Changing a template
  yields a different functor; changing a page yields a different object.
- **Emit:** Writes the final HTML string to the output filesystem.

**Compile errors (functor failure):**

Compile is a partial functor. It fails when cross-dependencies are invalid.

| Failure                | Formal meaning                                                                                       | Error                                        |
| :--------------------- | :--------------------------------------------------------------------------------------------------- | :------------------------------------------- |
| Broken inter-page link | `references` morphism in C has no `links to` image in O (target page doesn't produce an output file) | "Page `X` links to `Y` which does not exist" |
| Orphaned nav entry     | `nav child` morphism points to non-existent section                                                  | "Navigation references missing section `X`"  |
| Invalid tag reference  | `tagged with` morphism points to non-existent tag object                                             | "Tag `X` referenced but never defined"       |
| Render failure         | Catamorphism sub-function fails (e.g., invalid LaTeX, broken Mermaid)                                | "Failed to render math block in `X`"         |

---

### Composition: The Full Pipeline

The complete compiler is the composition of functors:

```
Compile ∘ Parse : S → O
```

**Correctness criterion:** The site is valid if and only if `Compile ∘ Parse` is
a well-defined functor: every object in S maps to an object in O, every
morphism in S maps to a morphism in O, and composition is preserved.

**"Done" criterion:** Sukr is complete when:

1. **C captures all valid site structures.** Every feature Sukr supports
   corresponds to objects and morphisms in C.
2. **Parse maps all valid Source trees to C.** No valid content directory is
   rejected.
3. **Compile maps all valid C to O.** No valid content model fails to produce
   output.
4. **Functor laws hold.** Composition and identity are preserved through both
   functors.
5. **Invalid inputs produce functor failures.** Every malformed site is caught
   as a well-defined error, not silent corruption.

## Validation

| Check                        | Result  | Detail                                                                                                                                                                                                                                                                                                                                                                    |
| :--------------------------- | :------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Olog labeling**            | PASS    | All Content objects use singular indefinite noun phrases. All morphisms are functional (each domain element maps to exactly one codomain element).                                                                                                                                                                                                                        |
| **Diagram commutativity**    | PASS    | Output path determination commutes (section path + page slug ↔ page output path). Nav derivation commutes (structure + weights → nav tree).                                                                                                                                                                                                                              |
| **Reference integrity**      | PASS    | `references` and `tagged with` morphisms are validated by functor well-definedness; no separate mechanism needed.                                                                                                                                                                                                                                                         |
| **Functor composition**      | PASS    | `Compile ∘ Parse` preserves composition: filesystem hierarchy maps to content hierarchy maps to output directory structure.                                                                                                                                                                                                                                               |
| **External adequacy**        | PASS    | The model captures pages, sections, navigation, tags, feeds, sitemaps, code highlighting, math, diagrams, 404 pages, aliases/redirects, and inter-page links. Static asset copying is explicitly out of scope (see Scope Boundary).                                                                                                                                       |
| **Minimality**               | PASS    | Every object and morphism in the model has a domain counterpart. No tensor products, braiding, coalgebras, or other machinery without domain justification.                                                                                                                                                                                                               |
| **Assumption independence**  | PASS    | The algebraic (constructor-based) framing was verified independently via the SDMA Decision Matrix and algebra/coalgebra duality analysis. No hidden state, no infinite behavior, no observer-based identity.                                                                                                                                                              |
| **Partial functor adequacy** | PARTIAL | Parse and Compile are modeled as partial functors (undefined on invalid inputs). Standard category theory uses total functors. The partiality is well-defined (each failure mode is enumerated), but a stricter formalization would use a result category or error monad. Acceptable for the current modeling purpose; the partiality captures error handling faithfully. |

## Implications

### Implementation Guidance

1. **Content category as type system.** The objects and morphisms of C correspond
   to Rust types and their relationships. `SiteManifest`, `Section`, `Page`,
   `Frontmatter`, `ContentBlock`, `NavItem`, `Tag`. Each should be a
   well-defined type. Morphisms map to fields or methods.

2. **Parse and Compile are the two module boundaries.** The codebase should
   have a clean separation between content discovery (Parse) and content
   rendering (Compile), with the Content types as the interface between them.

3. **Cross-dependency validation belongs in Parse, not Compile.** The
   `references` morphisms are discovered and validated during parsing, so
   that by the time Compile runs, Content is known to be well-formed.

4. **ContentBlock coproduct as interception pattern.** Each intercepted variant
   maps to a renderer. Adding a new block type (e.g., a new diagram language)
   means extending this coproduct: add the variant, add the renderer. `Prose`
   absorbs everything that does not require interception.

### Testing Strategy

1. **Functor law tests:** Verify that composition is preserved. If a file is
   nested `content/posts/hello.md`, the output should be at
   `public/posts/hello/index.html`. The hierarchy must be preserved through
   both functors.

2. **Reference integrity tests:** Create content with cross-page links and
   verify that broken links produce compile errors, not silent corruption.

3. **Parse failure tests:** Invalid frontmatter, missing delimiters, orphaned
   directories. Verify each enumerated failure mode.

4. **Catamorphism correctness tests:** Each ContentBlock variant renders
   correctly in isolation. This is the "unit test" level of the model.

### Architectural Invariants

1. **Link extraction during Parse.** Inter-page links are morphisms in C.
   They must be discovered and validated during Parse, before Compile runs.
   Broken internal references are compile-time errors (or warnings), never
   silent corruption in the output.

2. **Module boundary clarity.** Parse and Compile are the two phases. The
   Content types are the interface between them. No rendering logic should
   execute before Content is fully constructed; no parsing logic should
   execute during Compile.

3. **Tag as first-class object.** Tag has its own object status in C. Tags
   are typed values, not ad-hoc strings. The `tagged with` morphism is
   enforced at the type level.

### Open Questions

1. **Draft pages:** The current implementation supports `draft = true` in
   frontmatter, excluding pages from output. The model could represent this as a
   subobject classifier on Page, but for now it's handled by Parse simply not
   creating objects for drafts. Is this sufficient?

2. **Aliases/redirects:** URL aliases generate redirect HTML files. The model
   includes redirect files as Output objects, but the alias source (frontmatter
   field) isn't explicitly modeled as a morphism in C. Worth formalizing if
   aliases become more complex.

3. **Incremental compilation:** The model assumes batch compilation (total
   function from all inputs to all outputs). Incremental compilation would
   require tracking deltas (which objects/morphisms changed). This is a future
   concern, not a current one, but the functorial structure is compatible with
   it (functors compose with delta functors).
