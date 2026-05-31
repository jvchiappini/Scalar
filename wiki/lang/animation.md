# Animation Functions

Functions for animating lines and shapes.

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
| `easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

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
