---
title: Syntax Highlighting
description: Build-time code highlighting with Tree-sitter and tree-house
weight: 3
---

sukr highlights code blocks at build time using [tree-house](https://github.com/helix-editor/tree-house) (Helix editor's Tree-sitter integration). No client-side JavaScript required.

## Usage

Use fenced code blocks with a language identifier:

````md
```rust
fn main() {
    println!("Hello, world!");
}
```
````

## Supported Languages

| Language   | Identifier            |
| ---------- | --------------------- |
| Rust       | `rust`, `rs`          |
| Python     | `python`, `py`        |
| JavaScript | `javascript`, `js`    |
| TypeScript | `typescript`, `ts`    |
| Go         | `go`, `golang`        |
| Bash       | `bash`, `sh`, `shell` |
| Nix        | `nix`                 |
| TOML       | `toml`                |
| YAML       | `yaml`, `yml`         |
| JSON       | `json`                |
| HTML       | `html`                |
| CSS        | `css`                 |
| Markdown   | `markdown`, `md`      |
| C          | `c`                   |

## Examples

### Rust

```rust
fn main() {
    println!("Hello, world!");
}
```

### Python

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

### JavaScript

```javascript
const greet = (name) => `Hello, ${name}!`;
```

### TypeScript

```typescript
function greet(name: string): string {
  return `Hello, ${name}!`;
}
```

### Go

```go
func main() {
    fmt.Println("Hello, world!")
}
```

### Bash

```bash
#!/bin/bash
echo "Hello, $USER!"
```

### Nix

```nix
{ pkgs }:
pkgs.mkShell { buildInputs = [ pkgs.hello ]; }
```

### TOML

```toml
[package]
name = "sukr"
version = "0.1.0"
```

### YAML

```yaml
name: Build
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
```

### JSON

```json
{
  "name": "sukr",
  "version": "0.1.0"
}
```

### HTML

```html
<!DOCTYPE html>
<html>
  <body>
    Hello!
  </body>
</html>
```

### CSS

```css
.container {
  display: flex;
  color: #ff79c6;
}
```

### C

```c
#include <stdio.h>
int main() {
    printf("Hello!\n");
    return 0;
}
```

## How It Works

1. During Markdown parsing, code blocks are intercepted
2. tree-house parses the code and generates a syntax tree
3. Spans are generated with **hierarchical CSS classes** (e.g., `.hl-keyword-control-return`)
4. All work happens at build time—zero JavaScript in the browser

## Theme System

sukr uses a **decoupled theme system** with CSS custom properties. Themes are separate CSS files that define colors for syntax highlighting classes.

### Hierarchical Scopes

Highlighting uses fine-grained scope classes with hierarchical fallback:

| Scope Class                       | Description           |
| --------------------------------- | --------------------- |
| `.hl-keyword`                     | Generic keywords      |
| `.hl-keyword-control`             | Control flow          |
| `.hl-keyword-control-return`      | return/break/continue |
| `.hl-function`                    | Function names        |
| `.hl-function-builtin`            | Built-in functions    |
| `.hl-type`                        | Type names            |
| `.hl-variable`                    | Variables             |
| `.hl-variable-parameter`          | Function parameters   |
| `.hl-string`                      | String literals       |
| `.hl-comment`                     | Comments              |
| `.hl-comment-block-documentation` | Doc comments          |

If a theme only defines `.hl-keyword`, it will apply to all keyword subtypes.

### Using a Theme

Themes are CSS files that define the color palette. Import a theme at the top of your stylesheet:

```css
@import "path/to/theme.css";
```

sukr uses [lightningcss](https://lightningcss.dev/) which inlines `@import` rules at build time, producing a single bundled CSS file.

### Available Themes

sukr includes several themes in the `themes/` directory:

- **dracula.css** — Classic Dracula colors
- **gruvbox.css** — Warm retro palette
- **nord.css** — Cool arctic colors
- **github_dark.css** — GitHub's dark mode
- **github_light.css** — GitHub's light mode
- **snazzy.css** — Vibrant modern colors
- **catppuccin_mocha.css** — Warm pastel dark theme
- **tokyonight.css** — Japanese-inspired dark theme
- **rose_pine.css** — Elegant soho-inspired theme
- **onedark.css** — Classic Atom editor theme

Copy the theme files to your project and import as shown above.

### Core Variables

All themes define these CSS custom properties in `:root`:

| Variable        | Description            |
| --------------- | ---------------------- |
| `--hl-keyword`  | Keywords, control flow |
| `--hl-string`   | String literals        |
| `--hl-function` | Function names         |
| `--hl-comment`  | Comments               |
| `--hl-type`     | Type names             |
| `--hl-number`   | Numeric literals       |
| `--hl-variable` | Variables              |
| `--hl-operator` | Operators              |

### Customizing a Theme

Import a theme, then override specific variables in your own CSS:

```css
@import "themes/dracula.css";

/* Override just the keyword color */
:root {
  --hl-keyword: #e879f9;
}
```

Changing a variable updates every scope that references it.

### Theme Structure

Themes use CSS custom properties for easy customization:

```css
:root {
  --hl-keyword: #ff79c6;
  --hl-string: #f1fa8c;
  --hl-function: #50fa7b;
  --hl-comment: #6272a4;
}

.hl-keyword {
  color: var(--hl-keyword);
}
.hl-string {
  color: var(--hl-string);
}
/* ... */
```

## Injection Support

Some languages support **injection**—highlighting embedded languages. For example, bash inside Nix strings:

```nix
stdenv.mkDerivation {
  buildPhase = ''
    echo "Building..."
    make -j$NIX_BUILD_CORES
  '';
}
```

Markdown also supports injection—code blocks inside markdown fences are highlighted with their respective languages.

Languages with injection support: Bash, C, CSS, Go, HTML, JavaScript, Markdown, Nix, Python, Rust, TOML, TypeScript, YAML.

## Fallback

Unknown languages fall back to plain `<code>` blocks without highlighting.
