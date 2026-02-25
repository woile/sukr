+++
title = "Sections"
description = "Section types, frontmatter, and template dispatch reference"
weight = 2
+++

sukr discovers sections from your content directory structure. For an explanation of how sections work and how directories map to site structure, see [Content Organization](../content-organization.html).

## Section Types

The section type determines which template is used. It resolves in order:

1. **Frontmatter override**: `section_type = "blog"` in `_index.md`
2. **Directory name**: `content/blog/` → type `blog`

### Built-in Section Types

| Type       | Behavior                                               |
| ---------- | ------------------------------------------------------ |
| `blog`     | Sorts by date (newest first), renders individual posts |
| `projects` | Sorts by weight, card-style listing                    |
| _(other)_  | Sorts by weight, uses default template                 |

## Section Frontmatter

In `_index.md`:

```toml
+++
title = "My Blog"
description = "Thoughts and tutorials"
section_type = "blog"  # Optional, defaults to directory name
weight = 1             # Nav order
+++
```

## Adding a New Section

1. Create directory: `content/recipes/`
2. Create index: `content/recipes/_index.md`
3. Add content: `content/recipes/pasta.md`
4. Optionally create template: `templates/section/recipes.html`

If no `section/recipes.html` template exists, sukr falls back to `section/default.html`. A custom template is only needed when you want a different layout for that section.
