+++
title = "Math Rendering"
description = "Build-time LaTeX math with KaTeX"
weight = 5
+++

sukr renders LaTeX math expressions at build time using KaTeX, producing static HTML and CSS. No client-side JavaScript required.

## Inline Math

Use single dollar signs for inline math:

```markdown
The quadratic formula is $x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}$.
```

Renders as: The quadratic formula is $x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}$.

## Display Math

Use double dollar signs for display (block) math:

```markdown
$$
E = mc^2
$$
```

Or fence with `math` language:

````markdown
```math
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
```
````

## Supported Features

KaTeX supports a large subset of LaTeX math:

| Feature                 | Syntax                                         | Rendered                                       |
| ----------------------- | ---------------------------------------------- | ---------------------------------------------- |
| Greek letters           | `\alpha, \beta, \gamma`                        | $\alpha, \beta, \gamma$                        |
| Fractions               | `\frac{a}{b}`                                  | $\frac{a}{b}$                                  |
| Subscripts/superscripts | `x_i^2`                                        | $x_i^2$                                        |
| Summations              | `\sum_{i=1}^{n} i`                             | $\sum_{i=1}^{n} i$                             |
| Integrals               | `\int_a^b f(x)\,dx`                            | $\int_a^b f(x)\,dx$                            |
| Square roots            | `\sqrt{x^2 + y^2}`                             | $\sqrt{x^2 + y^2}$                             |
| Matrices                | `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` | $\begin{pmatrix} a & b \\ c & d \end{pmatrix}$ |

### Display Math Examples

The Gaussian integral:

$$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$

Euler's identity:

$$e^{i\pi} + 1 = 0$$

The Schrödinger equation:

$$i\hbar\frac{\partial}{\partial t}\Psi = \hat{H}\Psi$$

## How It Works

1. Math delimiters (`$...$`, `$$...$$`) are detected during parsing
2. KaTeX renders the expression to HTML + CSS
3. Required fonts are embedded inline
4. Output is pure HTML—no JavaScript

## Styling

KaTeX output uses semantic classes. Customize appearance:

```css
.katex {
  font-size: 1.1em;
}

.katex-display {
  margin: 1.5em 0;
  overflow-x: auto;
}
```

## Error Handling

Invalid LaTeX produces an error message inline rather than breaking the build:

```markdown
$\invalid{command}$
```

Renders with a red error indicator showing what went wrong.
