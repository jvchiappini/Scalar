# Text &amp; Import — SVG, Font, and Text Rendering

Three new functions for file import and vector text rendering:

| Function | Returns | Description |
|----------|---------|-------------|
| `SVGImport(path)` | `List[NodeId]` | Load an SVG file and render all `<path>` elements |
| `FontImport(path)` | `Number` (index) | Load a TrueType/OpenType font for text rendering |
| `Text(str, x, y [, ...kwargs])` | `NodeId` | Render a string as vector paths using a loaded font |

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
| `font` | Number | `0` | Font index from `FontImport()` |
| `size` | Number | `48` | Font size in pixels |
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
