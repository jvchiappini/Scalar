# Animation Functions

Functions for animating lines, shapes, and text.

---

## Quick Reference

| Function | Target | Effect |
|----------|--------|--------|
| `Animate` | Any node/list | Universal animation dispatcher with per-element stagger |
| `SetLineProgress` | Lines | Immediate line endpoint interpolation |
| `SetLineCap` | Lines | Change line cap style |
| `FadeIn` | Any node | Opacity 0 → 1 |
| `FadeOut` | Any node | Opacity 1 → 0 |
| `Grow` | Any node | Scale 0 → 1 |
| `Shrink` | Any node | Scale 1 → 0 |
| `MoveTo` | Any node | Position slide |
| `WriteText` | Text | Character-by-character fade-in reveal |
| `RevealText` | Text | Character-by-character draw-then-fill reveal |
| `DrawThenFill` | Any node | Scale up + fill fade-in two-phase animation |
| `Morph` | Any node with `PathData` | Path vertex interpolation between two shapes |

---

## Standard Animation Kwargs

All animation functions (except `SetLineProgress` and `SetLineCap`) accept these kwargs:

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `duration` | Number | `1.0` | Animation duration in seconds |
| `delay` | Number | `0.0` | Delay before animation starts |
| `easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

---

## SetLineProgress

```scalar
SetLineProgress(node_id, progress)
```

Interpolates a line endpoint so that `progress=0.0` collapses the line at its start point and `progress=1.0` draws the full segment.

| Arg | Type | Description |
|-----|------|-------------|
| `node_id` | NodeId / Number | Line to animate |
| `progress` | Number | Value in `[0, 1]` |

---

## SetLineCap

```scalar
SetLineCap(node_id, cap)
```

Changes the line cap style.

| Arg | Type | Description |
|-----|------|-------------|
| `node_id` | NodeId / Number | Target line |
| `cap` | String / Number | `"flat"` (0), `"round"` (1, default), `"square"` (2) |

---

## Animate — Universal Animation Dispatcher

The professional, Manim-style way to animate any node or list of nodes. `Animate()`
handles per-element stagger automatically — **no for-loops needed**.

### Basic Signatures

```scalar
// Single element animation
Animate(target, "FadeIn", duration: 1.0, delay: 0.0)
Animate(target, "FadeOut", duration: 1.0)
Animate(target, "Grow", duration: 1.0)
Animate(target, "Shrink", duration: 1.0)
Animate(target, "DrawThenFill", duration: 1.0, fill: [r,g,b,a])
Animate(target, "MoveTo", x: 100, y: 200, duration: 1.0)

// List with per-element stagger (like Manim's Write)
Animate(parts, "FadeIn", duration: 0.25, stagger: 0.08)

// Line-draw animation (backward compat — effect defaults to "LineDraw")
Animate(line_id, duration: 1.0)
Animate(lines: [id, ...], per_line: 0.5, staggered: true)
```

### Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `effect` | String | `"LineDraw"` | Animation effect name (2nd positional arg) |
| `duration` | Number | `1.0` | Animation duration in seconds |
| `delay` | Number | `0.0` | Global delay before animation starts |
| `stagger` | Number | `0.0` | Per-element delay increment (for list targets) |
| `easing` | String | `"ease_out_cubic"` | Easing function name |
| `fill` | [r,g,b,a] | — | Target fill color (required for `DrawThenFill`) |
| `x` | Number | — | Target x position (required for `MoveTo`) |
| `y` | Number | — | Target y position (required for `MoveTo`) |
| `into` | NodeId | — | Source node for morphing (required for `"Morph"`) |

### Effect Reference

| Effect | Extra Kwargs | Description |
|--------|-------------|-------------|
| `"FadeIn"` | — | Opacity 0 → 1 |
| `"FadeOut"` | — | Opacity 1 → 0 |
| `"Grow"` | — | Scale 0 → 1 |
| `"Shrink"` | — | Scale 1 → 0 |
| `"DrawThenFill"` | `fill: [r,g,b,a]` | Phase 1: scale up outline; Phase 2: fill fades in |
| `"MoveTo"` | `x: N, y: N` | Slide position from current → (x, y) |
| `"LineDraw"` | — | Line endpoint draw (for `Line()` objects) |
| `"Morph"` | `into: source_node` | Path vertex morph from source → target shape |

### Examples

```scalar
// === Per-glyph LaTeX Write (Manim-style) ===
// The 'stagger' kwarg adds per-element delay automatically
let parts = Tex("E = mc^2", x: 0, y: 0, fill: [0.3, 0.6, 1, 1])
Animate(parts, "FadeIn", duration: 0.25, stagger: 0.08, delay: 0.5)

// === DrawThenFill with stagger ===
let eq = Tex("\\int f(x) dx", x: 0, y: 0, fill: [1, 0.4, 0.4, 1])
Animate(eq, "DrawThenFill", duration: 0.4, stagger: 0.1, fill: [1, 0.4, 0.4, 1])

// === Simple shape ===
let ball = Circle(0, 0, 30, fill: [1, 0.8, 0, 1])
Animate(ball, "Grow", duration: 1.0, easing: "ease_out_back")
Animate(ball, "MoveTo", x: 200, y: 0, duration: 1.5, delay: 1.2)

// === Line draw (backward compat) ===
let a = Line(-300, 0, 300, 0, 20, fill: [1, 0, 0, 1])
let b = Line(0, -200, 0, 200, 20, fill: [0, 1, 0, 1])
Animate(lines: [a, b], per_line: 1.5, staggered: true)
```

---

## FadeIn

```scalar
FadeIn(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
FadeIn([node_id, ...], duration: 1.0, ...)
```

Animates a node's opacity from `0.0` → `1.0` (invisible to fully visible).

Accepts a single NodeId or a list via positional arg or `lines:` kwarg (following the same pattern as `Animate`).

### Example

```scalar
let t = Text("Hello", 0, 0, font: f, size: 36)
FadeIn(t, duration: 1.5, easing: "ease_out_sine")
```

---

## FadeOut

```scalar
FadeOut(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
```

Animates a node's opacity from `1.0` → `0.0` (visible to invisible).

### Example

```scalar
let t = Text("Goodbye", 0, 0, font: f, size: 36)
FadeOut(t, duration: 1.0, easing: "ease_in_cubic")
```

---

## Grow

```scalar
Grow(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
```

Animates a node's uniform scale from `0.0` → its current scale. Creates a "grow from nothing" effect.

### Example

```scalar
let c = Circle(0, 0, 50, fill: [1, 0.3, 0.3, 1])
Grow(c, duration: 1.0, easing: "ease_out_back")
```

---

## Shrink

```scalar
Shrink(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
```

Animates a node's uniform scale from its current size → `0.0`. The reverse of `Grow`.

### Example

```scalar
Shrink(c, duration: 0.8, easing: "ease_in_cubic")
```

---

## MoveTo

```scalar
MoveTo(node_id, x, y, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
```

Animates a node's position from its current position to the target `(x, y)`.

| Arg | Type | Description |
|-----|------|-------------|
| `node_id` | NodeId / Number | Node to move |
| `x` | Number | Target x position |
| `y` | Number | Target y position |

### Example

```scalar
let ball = Circle(200, 0, 30, fill: [1, 0.8, 0, 1])
MoveTo(ball, -200, 0, duration: 2.0, easing: "ease_in_out_cubic")
```

---

## WriteText

```scalar
WriteText(str, x, y, font: 0, size: 48, duration: 1.0, ...kwargs) -> [NodeId, ...]
```

Renders a text string **character by character**, with each character appearing sequentially via a fade-in animation — similar to Manim's `Write` effect.

Returns a **list** of NodeIds, one per rendered character.

### WriteText Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `font` | Number | `0` | Font index from `FontImport()` |
| `size` | Number | `48` | Font size in pixels |
| `duration` | Number | `1.0` | Total animation duration (all characters) |
| `per_char` | Number | — | Duration per character (overrides `duration`) |
| `delay` | Number | `0.0` | Global delay before the text starts appearing |
| `easing` | String | `"ease_out_cubic"` | Easing for character fade-in |
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
| `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness |
| `opacity` | Number | `1.0` | Global opacity |
| `z_index` | Number | `0` | Z-order |
| `rotation` | Number | `0` | Rotation in degrees |

### Example

```scalar
let f = FontImport("Roboto-Regular.ttf")
let chars = WriteText("Hello!", 0, 0,
    font: f, size: 48,
    duration: 2.0,
    delay: 1.0,         // wait 1s before starting
    easing: "ease_out_bounce",
    fill: [0.3, 0.6, 1, 1])
```

The returned list can be passed to `FadeOut()`, `Grow()`, etc. to animate the entire word:

```scalar
FadeOut(chars, duration: 1.0, easing: "ease_in_cubic")
```

### How It Works

Internally, `WriteText()`:
1. Loads the font and parses each character's glyph outline (same as `Text()`)
2. Spawns each character as a **separate** path entity with its own NodeId
3. Registers a staggered fade-in animation for each character, from left to right
4. Characters are hidden initially and revealed as the animation progresses

This means each character can be individually animated — you can fade out only some characters, grow individual letters, or move them independently.

---

## RevealText

```scalar
RevealText(str, x, y, font: 0, size: 48, duration: 1.0, ...kwargs) -> [NodeId, ...]
```

Renders text character-by-character with a **path-by-path draw-then-fill** reveal — similar to Manim's `DrawBorderThenFill`. For each character:

1. The outline path is broken into individual draw segments (each `LineTo`, `CubicTo`, `ClosePath`)
2. **Phase 1** (0–60% of eased progress): stroke segments appear one by one along the outline, like a pen tracing the character
3. **Phase 2** (60–100%): all segments are visible, fill fades in from transparent → original color

Returns a **list** of NodeIds, one per rendered character (the fill entity ID).

### RevealText Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `font` | Number | `0` | Font index from `FontImport()` |
| `size` | Number | `48` | Font size in pixels |
| `duration` | Number | `1.0` | Total animation duration (all characters) |
| `per_char` | Number | — | Duration per character (overrides `duration`) |
| `delay` | Number | `0.0` | Global delay before the text starts drawing |
| `easing` | String | `"ease_out_cubic"` | Easing for character draw-then-fill |
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
| `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness |
| `segment_subdivisions` | Number | `1` | Split each path segment into N sub-segments for smoother progressive drawing (4–8 recommended for fonts) |
| `opacity` | Number | `1.0` | Global opacity |
| `z_index` | Number | `0` | Z-order |
| `rotation` | Number | `0` | Rotation in degrees |

### Example

```scalar
let f = FontImport("Roboto-Regular.ttf")
let chars = RevealText("Hello!", 0, 0,
    font: f, size: 48,
    duration: 2.0,
    easing: "ease_out_bounce",
    fill: [0.3, 0.6, 1, 1],
    stroke: [0.2, 0.2, 0.2, 1],
    stroke_width: 1.5,
    segment_subdivisions: 6)  // smoother outline draw for thin characters
```

### How It Works

Internally, each character becomes **multiple entities**:
- One **fill entity** (full path, fill only, no stroke) — hidden initially
- N **stroke segment entities** (one per path segment, stroke only, no fill) — hidden initially

The `PathDrawThenFill` animation shows segments one by one during phase 1, then fades in the fill during phase 2.

---

## DrawThenFill

```scalar
DrawThenFill(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic", fill: [r,g,b,a])
```

Two-phase reveal animation for any node (shapes, text, etc.), inspired by Manim's `DrawBorderThenFill`:

- **Phase 1** (0–60% of eased progress): node scales from 0→1 with fill transparent
- **Phase 2** (60–100%): scale stays at 1, fill fades in from transparent → original color

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Target fill color to animate in |
| `duration` | Number | `1.0` | Duration |
| `delay` | Number | `0.0` | Delay before start |
| `easing` | String | `"ease_out_cubic"` | Easing function |

### Example

```scalar
let c = Circle(0, 0, 50, fill: [0.3, 0.6, 1, 1], stroke: [0.2, 0.2, 0.2, 1], stroke_width: 2)
DrawThenFill(c, duration: 1.5, fill: [0.3, 0.6, 1, 1], easing: "ease_out_back")
```

---

## Morph

```scalar
Morph(target, source, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
```

Animates the path of `target` so it morphs from `source`'s shape into `target`'s own shape.
Both nodes must have `PathData` (created by any shape function: `Circle`, `Rect`, `Polygon`,
`Tex()`, etc.).

### How It Works

1. All vertices and control points are extracted from both paths in traversal order
2. The shorter point sequence is padded with its last point to match lengths
3. Each frame, points are linearly interpolated and reconstructed as a polyline
   (`MoveTo` + `LineTo` + `Close`)
4. The reconstructed path replaces the target's `PathData` via `set_path_data()`

> ⚠️ Note: Curved segments become polylines during the morph. This is consistent with
> Manim-style morphing — visually smooth results for most shape-to-shape transitions.

### Using via Animate

```scalar
// Equivalent to Morph(target, source, ...)
Animate(target, "Morph", into: source, duration: 2.0)
```

### Examples

```scalar
// Basic shape morph
let circle = Circle(200, 0, 50, fill: [0.3, 0.6, 1, 1])
let rect = Rect(-200, 0, 120, 80, fill: [0.3, 0.6, 1, 1])
Morph(rect, circle, duration: 2.0)      // rect morphs from circle

// Staggered Tex morph
let eq1 = Tex("f(x) = x^2", x: 0, y: 0)
let eq2 = Tex("g(x) = \\sin x", x: 0, y: 0)
Animate(eq2, "Morph", into: eq1, duration: 1.0)

// Per-glyph text morph (Text() returns [NodeId], one per glyph)
let hello = Text("Hello", x: -100, y: 0, font: f, size: 48)
let world = Text("World", x: -100, y: 0, font: f, size: 48,
    fill: [0.6, 0.4, 1.0, 1.0])
SetVisibility(world, false)
Animate(world, "Morph", into: hello, duration: 1.2, easing: "ease_in_out_cubic")
// Each glyph morphs independently: H→W, e→o, l→r, l→l, o→d
```

---

## How Animations Work

All timed animations use the following lifecycle:
1. On the first frame where `time >= delay`, `start_time` is initialized
2. `progress = (time - start_time) / duration`, clamped to `[0, 1]`
3. If `was_hidden` is true, the element is shown on first `progress > 0`
4. The easing curve is applied to `progress`
5. The relevant property (opacity, scale, position, fill alpha) is interpolated
6. When `progress >= 1.0`, the animation is removed
