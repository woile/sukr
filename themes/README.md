# Syntax Highlighting Themes

This directory contains CSS themes for sukr's syntax highlighting system.

## Attribution

These themes are adapted from the [Helix editor](https://github.com/helix-editor/helix) theme collection.

Helix is licensed under the Mozilla Public License 2.0 (MPL-2.0). We are grateful to the Helix maintainers and contributors, as well as the original theme authors, for their excellent work.

## Available Themes

| Theme                  | Description                  |
| ---------------------- | ---------------------------- |
| `dracula.css`          | Classic Dracula colors       |
| `gruvbox.css`          | Warm retro palette           |
| `nord.css`             | Cool arctic colors           |
| `github_dark.css`      | GitHub's dark mode           |
| `github_light.css`     | GitHub's light mode          |
| `snazzy.css`           | Vibrant modern colors        |
| `catppuccin_mocha.css` | Warm pastel dark theme       |
| `tokyonight.css`       | Japanese-inspired dark theme |
| `rose_pine.css`        | Elegant soho-inspired theme  |
| `onedark.css`          | Classic Atom editor theme    |

## Theme Structure

Each theme defines CSS custom properties in `:root` and maps them to hierarchical `.hl-*` classes. Copy a theme to your project's static directory and import it in your CSS.

For usage, customization, and the full list of CSS variables, see the [Syntax Highlighting docs](https://sukr.io/features/syntax-highlighting.html).

## Note

These themes are **not bundled into the sukr binary** — they're provided as starting points. Copy what you need to your project and customize to match your site's design.
