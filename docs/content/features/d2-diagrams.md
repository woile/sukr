+++
title = "D2 Diagrams"
description = "Build-time diagram rendering with D2"
weight = 4
+++

sukr renders D2 diagrams at build time, producing inline SVG. No client-side JavaScript required.

## Usage

Use fenced code blocks with `d2` language:

````markdown
```d2
A -> B
direction: right
```
````

## Supported Diagram Types

It uses [d2-little](https://crates.io/crates/d2-little) which targets byte-exact SVG parity with the upstream Go d2.

## How It Works

1. During D2 parsing, `d2` code blocks are intercepted
2. The D2 definition is parsed and rendered to SVG
3. The SVG is inlined directly in the HTML output
4. No JavaScript or external resources needed

## Example

```d2
Start: "" {
  shape: circle
  width: 10
}
End: "" {
  shape: circle
  width: 10
}

Start -> Still
Still -> End

Still -> Moving
Moving -> Still
Moving -> Crash
Crash -> End
```

**Note**: The default direction is `down`, so it might render a big SVG, you can either use `direction: right` or set a max-height.

## CSS

D2 SVGs basic CSS styling:

```css
.d2-diagram {
    margin: 1.5rem 0;
    display: flex;
    justify-content: center;
}

.d2-diagram svg {
  display: block;
  max-width: 100%;
  height: auto;
}

.d2-diagram svg > rect:first-child {
  fill: transparent !important;
}
```

## Fallback

If a diagram fails to render (complex diagrams, syntax errors), the original code block is preserved with an error comment.
