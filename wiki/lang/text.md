# Text &amp; Import — SVG, Font, and Text Rendering

Functions for file import and vector text rendering:

| Function | Returns | Description |
|----------|---------|-------------|
| `SVGImport(path)` | `List[NodeId]` | Load an SVG file and render all `<path>` elements |
| `FontImport(path)` | `Number` (index) | Load a TrueType/OpenType font for text rendering |
| `Text(str, x, y [, ...kwargs])` | `NodeId` | Render a string as vector paths using a loaded font |

For animated text (character-by-character reveal), see [`WriteText`](animation.md#writetext) in the animation docs.

---

## SVGImport

```scalar
let paths = SVGImport("path/to/file.svg")
```

Reads an SVG file from disk, finds all `<path>` elements, parses their `d` attribute,
and renders each as a separate 2D path shape. Returns a list of NodeIds, one per
`<path>` element.

**Attribute Support:**
| SVG Attribute | Maps To |
|---------------|---------|
| `d` | Path commands (M, L, C, Q, Z, etc.) |
| `fill` | `fill` kwarg (supports `none`, hex `#rgb`/`#rrggbb`/`#rrggbbaa`, named CSS colors) |
| `stroke` | `stroke` kwarg (same color formats) |
| `stroke-width` | `stroke_width` kwarg |
| `opacity` | `opacity` kwarg |

**Color formats supported:**
- `none` / `transparent` → no fill/transparent
- `#rgb` → shorthand hex
- `#rrggbb` / `#rrggbbaa` → full hex
- Named: `black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `gray`, `silver`, `maroon`, `purple`, `navy`, `orange`, `pink`, `brown`

**Limitations:**
- Only `<path>` elements are rendered (no `<rect>`, `<circle>`, `<g>`, etc.)
- No CSS styling or external stylesheet support

---

## FontImport

```scalar
let my_font = FontImport("path/to/Roboto-Regular.ttf")
```

Loads a TrueType (.ttf) or OpenType (.otf) font file and returns a numeric index
that can be passed to `Text()`'s `font` kwarg. Multiple fonts can be loaded:

```scalar
let sans = FontImport("Roboto-Regular.ttf")
let serif = FontImport("NotoSerif-Regular.ttf")
Text("Hello", 0, 100, font: sans, size: 32)
Text("World", 0, 0, font: serif, size: 28)
```

---

## Text

```scalar
Text("Hello World", x, y, 
     font: 0, 
     size: 48,
     fill: [1, 1, 1, 1],
     ...)
```

Renders text as **vector paths** (not rasterized). The text is resolution-independent
and supports all the same kwargs as other shapes: fill, stroke, stroke_width,
rotation, opacity, z_index, visible, cap.

**Position:** `(x, y)` is the **baseline start** position — the bottom-left point
of the first line of text. This is the standard typographic convention:
descenders (g, j, p, q, y) extend below y, ascenders go above.

### Text Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `size` | Number | `48` | Font size in pixels (uses embedded KaTeX fonts internally) |
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color (use `NONE` for no fill) |
| `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness in pixels |
| `opacity` | Number | `1.0` | Global opacity (0.0–1.0) |
| `rotation` | Number | `0` | Rotation in degrees (counter-clockwise) |
| `z_index` | Number | `0` | Z-order (higher = on top) |
| `visible` | Boolean | `true` | Visibility |

### Examples

```scalar
// Load font
let f = FontImport("Roboto-Regular.ttf")

// Basic text with fill
Text("Hello Scalar!", 0, 200,
     font: f, size: 48,
     fill: [0.3, 0.6, 1, 1])

// Stroke-only text
Text("Outline", -100, 100,
     font: f, size: 36,
     fill: NONE,
     stroke: [1, 1, 1, 1],
     stroke_width: 1.5)

// Rotated text with fill + stroke
Text("Rotated", 150, 50,
     font: f, size: 28,
     fill: [1, 0.8, 0.2, 1],
     stroke: [1, 0.4, 0, 1],
     rotation: -15)

// Faded / semi-transparent text
Text("Faded", -150, -50,
     font: f, size: 32,
     fill: [1, 1, 1, 1],
     opacity: 0.4)

// Text with Spanish characters
Text("¿Cómo estás? ñoño", 0, -100,
     font: f, size: 32,
     fill: [0.5, 1, 0.5, 1])
```

### How It Works

Internally, `Text()`:
1. Retrieves the font parser from the bridge's font list
2. For each character, gets the glyph outline (MoveTo, LineTo, QuadTo commands)
3. Converts QuadTo (quadratic bézier) to CubicTo (cubic bézier) using the formula:
   - `C1 = Q0 + 2/3·(Q1 - Q0)`
   - `C2 = Q2 + 2/3·(Q1 - Q2)`
4. Positions glyphs by their advance width
5. Combines all glyph outlines into a single PathData
6. Spawns a single `spawn_2d_path` entity with fill/stroke/transform applied

This means text is **not rasterized** — it's pure vector geometry that can be
scaled, rotated, and styled like any other shape. The font file is parsed for
its TrueType outlines but no GPU atlas or MSDF texture is required.

---

## Tex — LaTeX Math Rendering

```scalar
let pieces = Tex("E = mc^2", x: 0, y: 0, ...kwargs)
```

Renders a LaTeX math expression as vector paths using **RaTeX**, a pure-Rust
KaTeX-compatible math layout engine with ~99.5% syntax coverage. No external
tools (latex, dvisvgm, LaTeX distribution) are required — 19 KaTeX fonts are
embedded at compile time.

**Return value:** `[NodeId]` — a List of `NodeId`s, **one per display item**
(glyph, fraction bar, radical, matrix bracket, etc.). Each piece is a separate
shape at its correct position within the formula. This enables per-glyph /
per-piece animation (like Manim's `Write`).

> **Note:** Unlike `Text()`, `Tex()` uses its own math fonts internally (the 19
> KaTeX TrueType fonts) — it does **not** require or use a `FontImport()` font.

The pipeline is:
1. **RaTeX** parses the LaTeX expression into an AST
2. **RaTeX** applies TeX-quality layout rules (superscript/fraction/radical positioning)
3. **RaTeX** produces a flat display list of positioned glyphs
4. **Our bridge** converts the display list to path commands using `ttf-parser`

### Tex Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `x` | Number | `0` | X position |
| `y` | Number | `0` | Y position |
| `size` | Number | `48` | Font size in pixels |
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
| `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness |
| `opacity` | Number | `1.0` | Global opacity |
| `rotation` | Number | `0` | Rotation in degrees |
| `z_index` | Number | `0` | Z-order |
| `visible` | Boolean | `true` | Visibility |

### Examples: Per-Glyph Animation (Manim-style Write)

```scalar
// FadeIn each piece of a formula sequentially
let parts = Tex("\\frac{a}{b} = c", x: 0, y: 100, size: 48)
for part in parts {
    FadeIn(part, duration: 0.2)
}
```

```scalar
// Grow each piece with staggered delay
let parts = Tex("\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}",
    x: 0, y: 0, size: 40,
    fill: [0.3, 0.6, 1, 1])
let delay = 0.0
for part in parts {
    FadeIn(part, duration: 0.15, delay: delay)
    delay = delay + 0.08
}
```

```scalar
// Piecewise coloring — animate different parts differently
let parts = Tex("\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}",
    x: 0, y: 0, size: 36)
// The integral sign fades in first
FadeIn(parts[0], duration: 0.3)
// The rest fade in slightly later
for i in 1..len(parts) {
    FadeIn(parts[i], duration: 0.2, delay: 0.3)
}
MoveTo(parts[-1], 100, 0, duration: 1.0)
```

### Basic Usage (Simple Display)

```scalar
// Simple equation — no font import needed!
Tex("E = mc^2", x: -200, y: 100,
    size: 48,
    fill: [0.3, 0.6, 1, 1])

// Summation
Tex("\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}",
    x: 200, y: -100,
    size: 40,
    fill: [0.2, 1, 0.4, 1])
```

### Caching

Each call to `Tex()` parses, lays out, and renders the expression from scratch.
Caching by expression string is not yet implemented (the full RaTeX pipeline
is fast enough for most use cases).

### Requirements

None. No LaTeX distribution, no external tools, no font import needed.
The 19 KaTeX TrueType fonts are embedded in the binary.

### Supported LaTeX constructs

RaTeX supports ~99.5% of KaTeX syntax, including:

| Category | Examples |
|----------|----------|
| Basic | Letters, numbers, `+`, `-`, `=`, `(`, `)`, `[`, `]` |
| Superscript/subscript | `x^2`, `x_{12}`, `x^{2}_{1}`, `x_{1}^{2}` |
| Fractions | `\frac{a}{b}`, `\tfrac`, `\dfrac`, `\cfrac` |
| Square roots | `\sqrt{x}`, `\sqrt[3]{x}` |
| Large operators | `\sum`, `\int`, `\prod`, `\bigcup`, `\bigcap`, `\oint` with limits |
| Greek letters | `\alpha`, `\beta`, `\pi`, `\Gamma`, `\Delta`, `\Omega`… |
| Math symbols | `\infty`, `\to`, `\partial`, `\times`, `\cdot`, `\pm`, `\neq`, `\approx`, `\leq`, `\geq`, `\cap`, `\cup`, `\subset`, `\in`, `\forall`, `\exists`… |
| Functions | `\sin`, `\cos`, `\tan`, `\log`, `\ln`, `\lim`, `\det`, `\exp`, `\max`, `\min` |
| Brackets | `\left(`, `\right)`, `\left[`, `\right]`, `\left\{`, `\right\}`, `\vert`, `\langle`, `\rangle` |
| Text | `\text{inline text}` |
| Matrices | `\begin{matrix}`, `\begin{pmatrix}`, `\begin{bmatrix}`, `\begin{vmatrix}`, `\begin{cases}` |
| Aligned equations | `\begin{aligned}`, `\begin{gathered}`, `\begin{align}` |
| Accents | `\hat{x}`, `\tilde{x}`, `\bar{x}`, `\vec{x}`, `\dot{x}`, `\ddot{x}`, `\breve{x}` |
| Font commands | `\mathbf`, `\mathcal`, `\mathfrak`, `\mathbb`, `\mathrm`, `\mathit`, `\mathsf`, `\mathtt` |
| Color | `\color{red}{x}`, `\colorbox{yellow}{text}`, `\textcolor` |
| Styling | `\displaystyle`, `\textstyle`, `\scriptstyle`, `\scriptscriptstyle` |
| Under/Over | `\overline`, `\underline`, `\overbrace`, `\underbrace`, `\overrightarrow` |
| Spacing | `\;`, `\:`, `\,`, `\!`, `\quad`, `\qquad` |
| Chemistry | `\ce{H2SO4}`, `\pu{1.5e-3 mol//L}` (via mhchem) |
| Proof trees | `\begin{prooftree}` (bussproofs-style) |
