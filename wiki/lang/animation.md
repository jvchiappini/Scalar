# Animation Functions

Functions for animating lines, shapes, and text.

---

## Quick Reference

| Function | Target | Effect |
|----------|--------|--------|
| `Animate` | Lines | Line-draw reveal (progress 0→1) |
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

## Animate

```scalar
Animate(lines: [id, ...], per_line: 1.0, staggered: true, easing: "ease_out_cubic")
```

Registers a line-draw animation with easing. Each line draws from start to end over the specified duration. When `staggered` is true (default), lines animate sequentially — each one starts after the previous one finishes.

### Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `lines` | `[NodeId]` | (required) | Lines to animate |
| `per_line` | Number | `1.0` | Duration per line (seconds) |
| `duration` | Number | — | Total duration (alternative to `per_line`) |
| `staggered` | Boolean | `true` | If `true`, lines animate sequentially |
| `easing` | String | `"ease_out_cubic"` | Easing function name |

### Examples

```scalar
// Single line
let l = Line(-200, 0, 200, 0, 10, cap: "square")
Animate(lines: [l], per_line: 2.0)

// Multiple sequential lines
let a = Line(-300, 0, 300, 0, 20, 1, 0, 0)
let b = Line(0, -200, 0, 200, 20, 0, 1, 0)
Animate(lines: [a, b], per_line: 1.5, staggered: true)

// Multiple parallel lines
Animate(lines: [a, b], per_line: 2.0, staggered: false, easing: "ease_out_bounce")
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

Renders text character-by-character with a **"draw then fill"** reveal — similar to Manim's `DrawBorderThenFill`. Each character first scales from 0→1 (stroke visible, fill transparent), then fill fades from transparent to its original color.

Returns a **list** of NodeIds, one per rendered character.

### RevealText Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `font` | Number | `0` | Font index from `FontImport()` |
| `size` | Number | `48` | Font size in pixels |
| `duration` | Number | `1.0` | Total animation duration (all characters) |
| `per_char` | Number | — | Duration per character (overrides `duration`) |
| `easing` | String | `"ease_out_cubic"` | Easing for character draw-then-fill |
| `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
| `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness |
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
    stroke_width: 1.5)
```

The returned list can be chained with other animations:

```scalar
// Fade out the entire revealed text
FadeOut(chars, duration: 1.0, easing: "ease_in_cubic")
```

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

## How Animations Work

All timed animations use the following lifecycle:
1. On the first frame where `time >= delay`, `start_time` is initialized
2. `progress = (time - start_time) / duration`, clamped to `[0, 1]`
3. If `was_hidden` is true, the element is shown on first `progress > 0`
4. The easing curve is applied to `progress`
5. The relevant property (opacity, scale, position, fill alpha) is interpolated
6. When `progress >= 1.0`, the animation is removed
