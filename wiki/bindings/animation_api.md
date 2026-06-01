# Animation API (Bridge Bindings)

The Scalar bridge exposes animation functions for lines, shapes, and text.
All of them operate on `NodeId` values returned by `Line()`, `Rect()`, `Circle()`, `Text()`, etc.

---

## Common animation parameters

All timed animations accept the following keyword arguments:

| Parameter  | Type      | Default              | Description                              |
|------------|-----------|----------------------|------------------------------------------|
| `duration` | `Number`  | `1.0`                | Duration in seconds                      |
| `delay`    | `Number`  | `0.0`                | Delay before the animation starts        |
| `easing`   | `String`  | `"ease_out_cubic"`   | Easing curve (see `wiki/api/easing.md`)  |

---

## SetLineProgress(node_id, progress)

Updates a line so that only the first `progress` fraction is visible.
Progress `0.0` collapses the line at its start point; `1.0` draws the full segment.

**Arguments:**

| Argument   | Type     | Description                                     |
|------------|----------|-------------------------------------------------|
| `node_id`  | `NodeId` | The line to modify                              |
| `progress` | `Number` | Fraction to show, clamped to `[0.0, 1.0]`      |

**Example:**
```scalar
let l = Line(-100, 0, 100, 0, 5)
SetLineProgress(l, 0.5)  // muestra solo la mitad izquierda
```

---

## Animate(lines:, per_line:, staggered:)

Registers a line-draw animation that runs over the given duration (in seconds).

**Arguments (keyword):**

| Argument    | Type            | Default | Description                                            |
|-------------|-----------------|---------|--------------------------------------------------------|
| `lines`     | `[NodeId, ...]` | —       | One or more node IDs to animate (required)             |
| `per_line`  | `Number`        | `1.0`   | Duration per line in seconds                           |
| `staggered` | `Boolean`       | `true`  | `true` = sequential; `false` = all at once             |
| `easing`    | `String`        | `"ease_out_cubic"` | Easing function                               |

When `staggered` is `true` (default), each line starts drawing after the previous
one finishes, creating a sequential reveal effect. When `false`, all lines animate
in parallel.

**Examples:**
```scalar
let a = Line(-200, 0, 200, 0, 10, 1, 0, 0)
let b = Line(0, -150, 0, 150, 10, 0, 1, 0)

// Sequential: a draws (1s), then b draws (1s)
Animate(lines: [a, b], per_line: 1.0, staggered: true)

// Parallel: both draw simultaneously (1s)
Animate(lines: [a, b], per_line: 1.0, staggered: false)
```

---

## SetLineCap(node_id, cap)

Changes the cap style of an existing line.

**Arguments:**

| Argument   | Type            | Description                                            |
|------------|-----------------|--------------------------------------------------------|
| `node_id`  | `NodeId`        | The line to modify                                     |
| `cap`      | `String`/`Number` | `"round"` (1), `"square"` (2), or `"flat"` (0)      |

**Example:**
```scalar
let l = Line(-100, 0, 100, 0, 10)
SetLineCap(l, "square")
```

---

## FadeIn(node_id, duration, delay, easing)

Animates a node's opacity from `0.0` → `1.0` (invisible to fully visible).

**Arguments:**

| Argument   | Type            | Default              | Description                      |
|------------|-----------------|----------------------|----------------------------------|
| `node_id`  | `NodeId`        | —                    | Node to animate                  |
| `duration` | `Number` kwarg  | `1.0`                | Duration in seconds              |
| `delay`    | `Number` kwarg  | `0.0`                | Delay before start               |
| `easing`   | `String` kwarg  | `"ease_out_cubic"`   | Easing function                  |

**Example:**
```scalar
let t = Text("Hello", 0, 0, font: f, size: 36)
FadeIn(t, duration: 1.5, easing: "ease_out_sine")
```

---

## FadeOut(node_id, duration, delay, easing)

Animates a node's opacity from `1.0` → `0.0`.

Same parameters as `FadeIn`.

---

## Grow(node_id, duration, delay, easing)

Animates a node's uniform scale from `0.0` → its current scale.

**Example:**
```scalar
let c = Circle(0, 0, 50, fill: [1, 0.3, 0.3, 1])
Grow(c, duration: 1.0, easing: "ease_out_back")
```

---

## Shrink(node_id, duration, delay, easing)

Animates a node's uniform scale from its current value → `0.0`.

---

## MoveTo(node_id, x, y, duration, delay, easing)

Animates a node's position from its current position → `(x, y)`.

**Arguments:**

| Argument   | Type            | Default              | Description                      |
|------------|-----------------|----------------------|----------------------------------|
| `node_id`  | `NodeId`        | —                    | Node to move (positional)        |
| `x`        | `Number`        | —                    | Target x (positional or kwarg)   |
| `y`        | `Number`        | —                    | Target y (positional or kwarg)   |
| `duration` | `Number` kwarg  | `1.0`                | Duration in seconds              |
| `delay`    | `Number` kwarg  | `0.0`                | Delay before start               |
| `easing`   | `String` kwarg  | `"ease_out_cubic"`   | Easing function                  |

**Example:**
```scalar
MoveTo(ball, -200, 0, duration: 2.0, easing: "ease_in_out_cubic")
```

---

## DrawThenFill(node_id, duration, delay, easing, fill:)

Two-phase reveal animation for any node. Phase 1 (0–60% eased progress) scales
the node from 0→1 with fill transparent; Phase 2 (60–100%) holds scale at 1
and fades fill in from transparent → original color.

**Arguments:**

| Argument       | Type            | Default              | Description                          |
|----------------|-----------------|----------------------|--------------------------------------|
| `node_id`      | `NodeId`        | —                    | Node to animate (positional or list) |
| `fill`         | `[r,g,b,a]` kwarg | `[1,1,1,1]`        | Target fill color                    |
| `duration`     | `Number` kwarg  | `1.0`                | Duration in seconds                  |
| `delay`        | `Number` kwarg  | `0.0`                | Delay before start                   |
| `easing`       | `String` kwarg  | `"ease_out_cubic"`   | Easing function                      |

**Example:**
```scalar
let c = Circle(0, 0, 50, fill: [0.3, 0.6, 1, 1], stroke: [0.2, 0.2, 0.2, 1], stroke_width: 2)
DrawThenFill(c, duration: 1.5, fill: [0.3, 0.6, 1, 1], easing: "ease_out_back")
```

---

## WriteText(str, x, y, font:, size:, ...kwargs)

Renders text character-by-character with sequential fade-in animation.
Each character is a separate `NodeId` — returns a `List` of NodeIds.

**Arguments:**

| Argument       | Type            | Default              | Description                          |
|----------------|-----------------|----------------------|--------------------------------------|
| `str`          | `String`        | —                    | Text to render (positional)          |
| `x`            | `Number`        | —                    | Baseline x (positional or kwarg)     |
| `y`            | `Number`        | —                    | Baseline y (positional or kwarg)     |
| `font`         | `Number` kwarg  | `0`                  | Font index from `FontImport()`       |
| `size`         | `Number` kwarg  | `48`                 | Font size in pixels                  |
| `duration`     | `Number` kwarg  | `1.0`                | Total animation duration             |
| `per_char`     | `Number` kwarg  | —                    | Per-character duration (overrides `duration`) |
| `delay`        | `Number` kwarg  | `0.0`                | Global delay before the text starts appearing |
| `easing`       | `String` kwarg  | `"ease_out_cubic"`   | Easing for character reveal          |

**Returns:** `List[NodeId]` — one NodeId per rendered character.

**Example:**
```scalar
let f = FontImport("Roboto-Regular.ttf")
let chars = WriteText("Animated!", 0, 0,
    font: f, size: 48,
    duration: 2.0,
    easing: "ease_out_bounce")
```

---

## RevealText(str, x, y, font:, size:, ...kwargs)

Renders text character-by-character with a **path-by-path draw-then-fill** reveal
(Manim's `DrawBorderThenFill` style). For each character, the outline path is
broken into individual stroke segments that appear one by one (phase 1), then
the fill fades in (phase 2). Returns a `List` of NodeIds (fill entity IDs).

Supports the same kwargs as `WriteText`, plus the additional kwarg:

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `segment_subdivisions` | `Number` | `1` | Split each path segment into N sub-segments for smoother progressive drawing (4–8 recommended for fonts) |

**Example:**
```scalar
let f = FontImport("Roboto-Regular.ttf")
let chars = RevealText("Reveal!", 0, 0,
    font: f, size: 48,
    duration: 2.0,
    fill: [0.3, 0.6, 1, 1],
    stroke: [0.2, 0.2, 0.2, 1],
    stroke_width: 1.5,
    segment_subdivisions: 6)  // smoother outline for thin chars
```

**How it works internally per character:**
1. The glyph path is split into individual draw segments via `extract_path_segments`
2. Segments are optionally subdivided into N sub-segments via `subdivide_segments` for smoother progressive drawing
3. A **fill entity** is spawned (full path, fill only, no stroke) — hidden
4. For each sub-segment, a **stroke segment entity** is spawned (segment path, stroke only, no fill) — hidden
5. `AnimationKind::PathDrawThenFill` manages progressive segment reveal + fill fade

---

## DrawThenFill(node_id, duration, delay, easing, fill:)

Two-phase reveal animation for any node. Phase 1 (0–60% eased progress) scales
the node from 0→1 with fill transparent; Phase 2 (60–100%) holds scale at 1
and fades fill in from transparent → original color.

Unlike `RevealText` which does path-by-path stroke drawing, this is a simpler
scale-based animation suitable for any existing node.

---

## Implementation notes

- All animations run inside the engine's `before_frame` hook, called by `scalar_cli`
  once per frame before `render_frame(time)`.
- Each `AnimationEntry` stores `node_id`, `duration`, `delay`, `start_time`, `easing`,
  and an `AnimationKind` enum variant.
- On the first frame, `start_time` is set to `time + delay`.
- For `Scale`, `MoveTo`, and `DrawThenFill` animations, the initial transform is
  captured lazily from the ECS on the first frame.
- `DrawThenFill` also sets the fill to fully transparent on first frame and
  restores it with interpolated alpha during phase 2.
- `PathDrawThenFill` (used by `RevealText`) spawns separate stroke segment entities
  per character and shows them progressively during phase 1, then fades fill in
  during phase 2. Segments appear in path traversal order.
- Completed animations are automatically removed from the active list.
