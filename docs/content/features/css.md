+++
title = "CSS Minification"
description = "Automatic CSS optimization at build time"
weight = 7
+++

sukr automatically minifies CSS files in your static directory during the build.

## How It Works

When copying files from `static/` to your output directory:

1. CSS files (`.css` extension) are processed with lightningcss
2. Whitespace, comments, and redundant rules are removed
3. Identical selectors are merged
4. Other static files are copied unchanged

## Build Output

You'll see minification progress during builds:

```text
minifying: static/style.css → public/style.css (2048 → 1234 bytes)
copying: static/logo.svg → public/logo.svg
```

## No Configuration

Minification is always on. There's no setting to disable it, so if you need the original CSS, check your source files.

## Error Handling

If CSS parsing fails (malformed input), sukr preserves the original file content instead of failing the build. Check your terminal for warnings.

## What Gets Minified

- Whitespace and newlines removed
- Comments stripped
- Selector merging (`.a { color: red } .b { color: red }` → `.a, .b { color: red }`)
- Vendor prefix optimization

## What Doesn't Change

- CSS variable names
- Class and ID selectors
- Relative paths (images, fonts)
