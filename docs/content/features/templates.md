+++
title = "Tera Templates"
description = "Customizable templates without recompilation"
weight = 1
+++

sukr uses [Tera](https://keats.github.io/tera/), a Jinja2-like templating engine. Templates are loaded at runtime, so you can modify them without recompiling sukr. See the [Tera documentation](https://keats.github.io/tera/docs/) for template authoring syntax (filters, blocks, inheritance).

## Template Directory Structure

```text
templates/
├── base.html               # Shared layout (required)
├── page.html               # Standalone pages and homepage
├── section/
│   ├── default.html        # Fallback section index
│   ├── blog.html           # Blog section index
│   └── projects.html       # Projects section index
├── content/
│   ├── default.html        # Fallback content page
│   └── post.html           # Blog post
└── tags/
    └── default.html        # Tag listing page
```

## Template Inheritance

All templates extend `base.html`:

```html
{% extends "base.html" %} {% block content %}
<article>
  <h1>{{ page.title }}</h1>
  {{ content | safe }}
</article>
{% endblock content %}
```

`base.html` defines two overridable blocks: `{% block title %}` for the HTML `<title>` tag, and `{% block content %}` for the page body.

## Available Context Variables

### All Templates

| Variable            | Description                         |
| ------------------- | ----------------------------------- |
| `config.title`      | Site title                          |
| `config.author`     | Site author                         |
| `config.nav.nested` | Whether hierarchical nav is enabled |
| `nav`               | Array of navigation items           |
| `page_path`         | Current page path                   |
| `prefix`            | Relative path prefix for assets     |
| `config.base_url`   | Canonical base URL                  |
| `title`             | Current page title                  |

Each nav item has:

- `label` — Display text
- `path` — URL path
- `weight` — Sort order
- `children` — Child nav items (when `config.nav.nested` is true)

### Page Templates

| Variable           | Description                          |
| ------------------ | ------------------------------------ |
| `page.title`       | Page title                           |
| `page.description` | Page description                     |
| `page.toc`         | Whether TOC is enabled for this page |
| `content`          | Rendered HTML content                |
| `anchors`          | Array of heading anchors for TOC     |

Each anchor in `anchors` has:

- `id` — Heading slug (for `href="#id"`)
- `label` — Heading text
- `level` — Heading level (2-6, h1 excluded)

### Section Templates

| Variable              | Description                       |
| --------------------- | --------------------------------- |
| `section.title`       | Section title                     |
| `section.description` | Section description               |
| `items`               | Array of content items in section |

### Content Item Fields (in `items`)

| Variable           | Description         |
| ------------------ | ------------------- |
| `item.title`       | Content title       |
| `item.description` | Content description |
| `item.date`        | Publication date    |
| `item.path`        | URL path            |
| `item.slug`        | URL slug            |

### Tag Templates

| Variable | Description                          |
| -------- | ------------------------------------ |
| `tag`    | The tag name                         |
| `items`  | Array of content items with this tag |

## Template Override

Set `template` in frontmatter to use a custom template:

```toml
+++
title = "Special Page"
template = "special"
+++
```

This uses `templates/content/special.html` instead of the default.
