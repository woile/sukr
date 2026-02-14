+++
title = "Comparison"
description = "How sukr compares to other static site generators"
weight = 10
+++

This page provides a factual comparison of sukr with other popular static site generators.

## Feature Matrix

| Feature                   |      sukr       |  Zola   |     Hugo     |    Eleventy     |
| :------------------------ | :-------------: | :-----: | :----------: | :-------------: |
| **Language**              |      Rust       |  Rust   |      Go      |     Node.js     |
| **Single Binary**         |       ✅        |   ✅    |      ✅      |       ❌        |
| **Syntax Highlighting**   |   Tree-sitter   | syntect |    Chroma    |  Plugin-based   |
| **Build-time Math**       | ✅ KaTeX→MathML |   ❌    |      ❌      | Plugin required |
| **Build-time Diagrams**   | ✅ Mermaid→SVG  |   ❌    |      ❌      | Plugin required |
| **JS-Free Rich Content**¹ |       ✅        |   ❌    |      ❌      |  Configurable   |
| **Template Engine**       |      Tera       |  Tera   | Go templates |    Multiple     |

¹ _All generators can produce JS-free HTML for basic content. This row refers to built-in math and diagram rendering without client-side JavaScript. Zola and Hugo require external JS libraries (MathJax, Mermaid.js) for these features._

## Syntax Highlighting

**sukr** uses [Tree-sitter](https://tree-sitter.github.io/), the same parsing technology used by GitHub, Neovim, and Helix. Tree-sitter builds actual syntax trees rather than matching regex patterns, which enables:

- Accurate highlighting of edge cases
- Language injection (e.g., bash inside Nix `buildPhase`, JS inside HTML)
- Consistent results across all supported languages

**Zola** uses syntect, which is regex-based (Sublime Text grammars). It works well for common cases but can struggle with nested languages or unusual syntax.

**Hugo** uses Chroma, a Go port of Pygments. Similar trade-offs to syntect.

## Math Rendering

**sukr** renders LaTeX math to MathML at build time using KaTeX. The output is browser-native — no JavaScript required in the browser. Modern browsers render MathML directly.

**Zola, Hugo, Eleventy** typically require client-side JavaScript (MathJax or KaTeX.js) to render math, or external tooling pipelines.

## Diagram Rendering

**sukr** converts Mermaid diagram definitions to inline SVG at build time. The diagrams are embedded directly in the HTML — no JavaScript library loads in the browser.

**Other generators** typically include the Mermaid.js library and render diagrams client-side, adding ~1MB to page weight and requiring JavaScript.

## When to Choose sukr

Consider sukr if you:

- Want zero JavaScript in your output
- Need accurate syntax highlighting with language injection
- Prefer a single Rust binary with no runtime dependencies
- Value build-time rendering over client-side hydration

## When to Choose Something Else

Consider Zola, Hugo, or Eleventy if you:

- Need a larger plugin ecosystem
- Require features sukr doesn't have (taxonomies, i18n, etc.)
- Prefer a more established community with extensive themes
- Don't care about client-side JavaScript

sukr is intentionally minimal. It does a few things well rather than trying to cover every use case.
