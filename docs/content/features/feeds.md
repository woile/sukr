+++
title = "Atom Feeds"
description = "Build-time feed generation for blog posts"
weight = 6
+++

sukr generates an Atom 1.0 feed for blog posts at build time.

## Output

After building, you'll find `feed.xml` in your output directory:

```text
public/
├── index.html
├── feed.xml  ← Atom feed
└── blog/
    └── ...
```

## Feed Contents

The feed includes:

- Site title and author from `site.toml`
- Self-referencing links (required by Atom spec)
- Entry for each content item in `blog/` section
- Post title, URL, date, and description

## Auto-generation

Feed generation happens automatically when any content exists in a section with `section_type = "blog"`. No configuration required.

Posts are sorted by date (newest first), matching the blog section ordering.

## Linking to the Feed

Add a link in your `base.html` template:

```html
<link
  rel="alternate"
  type="application/atom+xml"
  title="{{ config.title }} Feed"
  href="{{ prefix }}/feed.xml"
/>
```

## Date Format

Post dates in frontmatter should use `YYYY-MM-DD` format:

```toml
+++
title = "My Post"
date = 2024-01-15
+++
```

The feed converts this to RFC 3339 format required by Atom.

## Validation

Test your feed with the [W3C Feed Validator](https://validator.w3.org/feed/).
