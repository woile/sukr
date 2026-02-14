+++
title = "Sitemap"
description = "Automatic XML sitemap generation for SEO"
weight = 7
+++

sukr generates an XML sitemap at build time for search engine optimization.

## Output

After building, you'll find `sitemap.xml` in your output directory:

```text
public/
├── index.html
├── feed.xml
├── sitemap.xml  ← XML sitemap
└── blog/
    └── ...
```

## Sitemap Contents

The sitemap includes URLs for:

- Homepage (`/index.html`)
- Section index pages (`/blog/index.html`, etc.)
- All content items within sections
- Standalone pages (top-level `.md` files)

## Auto-generation

Sitemap generation happens automatically during every build. No configuration required.

URLs use the `base_url` from `site.toml` to construct absolute URLs as required by the sitemap protocol.

## Last Modified Dates

If content has a `date` field in frontmatter, it's included as `<lastmod>`:

```toml
+++
title = "My Post"
date = 2024-01-15
+++
```

Content without dates omits the `<lastmod>` element.

## Linking to the Sitemap

Add a link in your `base.html` template or `robots.txt`:

```text
Sitemap: https://example.com/sitemap.xml
```

## Validation

Test your sitemap with [Google's Rich Results Test](https://search.google.com/test/rich-results) or the [XML Sitemap Validator](https://www.xml-sitemaps.com/validate-xml-sitemap.html).
