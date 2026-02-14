+++
title = "Security"
description = "Content trust model and deployment security guidance"
weight = 90
+++

# Security

sukr is a **build-time only** compiler with no runtime attack surface. Security considerations focus on content processing and deployment.

## Trust Model

| Source               | Trust Level      | Rationale                                            |
| :------------------- | :--------------- | :--------------------------------------------------- |
| Markdown content     | **Untrusted**    | May come from contributors, CMS, or external sources |
| TOML frontmatter     | **Untrusted**    | Parsed from content files                            |
| Templates            | **Semi-trusted** | User-controlled but typically from known sources     |
| sukr binary          | **Trusted**      | Compiled from audited Rust code                      |
| Tree-sitter grammars | **Trusted**      | Compiled into the binary                             |

## Content Processing

### HTML Passthrough

Per the CommonMark specification, raw HTML in Markdown is passed through to output:

```markdown
<script>alert('hello')</script>
```

**If your content comes from untrusted sources**, review it before building. sukr does not sanitize HTML — this is intentional to preserve legitimate use cases.

### URL Escaping

Link and image URLs are escaped to prevent attribute injection attacks:

```markdown
<!-- This is safe — quotes are escaped -->

[click me](<"%3E%3Cscript%3Ealert(1)%3C/script%3E>)
```

Produces escaped output, not executable script.

### Template Auto-Escaping

Tera templates auto-escape variables by default:

- `{{ title }}` — escaped (safe)
- `{{ page.description }}` — escaped (safe)
- `{{ content | safe }}` — intentionally unescaped (pre-rendered HTML)

## Deployment Security

### Content Security Policy

For maximum protection when serving sukr-generated sites, configure CSP headers on your web server or CDN.

**Recommended policy for sukr sites:**

```
Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; script-src 'none'; frame-ancestors 'none'
```

This policy:

- ✅ Allows styles (including inline for syntax highlighting)
- ✅ Allows images and data URIs (for Mermaid SVGs)
- ✅ Blocks all JavaScript execution
- ✅ Prevents clickjacking

### Platform-Specific Headers

**Cloudflare Pages** (`public/_headers`):

```
/*
  Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; script-src 'none'
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
```

**Netlify** (`public/_headers`):

```
/*
  Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; script-src 'none'
  X-Content-Type-Options: nosniff
```

**Nginx**:

```nginx
add_header Content-Security-Policy "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; script-src 'none'";
add_header X-Content-Type-Options nosniff;
add_header X-Frame-Options DENY;
```

## Reporting Issues

Report security issues via [security@sukr.io](mailto:security@sukr.io) or GitHub Security Advisories.
