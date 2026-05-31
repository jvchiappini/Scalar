# Animation API (Bridge Bindings)

The Scalar bridge exposes three functions for line-draw animation and cap styling.
All of them operate on `NodeId` values returned by `Line()`, `Rect()`, `Circle()`.

---

## SetLineProgress(node_id, progress)

Updates a line so that only the first `progress` fraction is visible.
Progress `0.0` collapses the line at its start point; `1.0` draws the full segment.

**Arguments:**

| Argument   | Type     | Description                                     |
|------------|----------|-------------------------------------------------|
| `node_id`  | `NodeId` | The line to animate                             |
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

When `staggered` is `true` (default), each line starts drawing after the previous
one finishes, creating a sequential reveal effect. When `false`, all lines animate
in parallel.

**Example:**
```scalar
let a = Line(-200, 0, 200, 0, 10, 1, 0, 0)
let b = Line(0, -150, 0, 150, 10, 0, 1, 0)

// Secuencial: a se dibuja (1s), luego b (1s)
Animate(lines: [a, b], per_line: 1.0, staggered: true)

// Paralelo: ambos se dibujan simultáneamente (1s)
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
SetLineCap(l, "square")  // puntas cuadradas
```

---

## Line cap kwarg on creation

The `Line()` function also accepts a `cap:` keyword argument at creation time:

```scalar
Line(-100, 0, 100, 0, 10, cap: "round")   // redondeadas (default)
Line(-100, 0, 100, 0, 10, cap: "square")  // cuadradas
Line(-100, 0, 100, 0, 10, cap: "flat")    // planas
```

---

## Implementation notes

- The animation runs inside the engine's `before_frame` hook, called by `scalar_cli`
  once per frame before `render_frame(time)`.
- Each `AnimatingLine` stores `node_id`, `duration`, `delay`, and `start_time`.
- On the first frame, `start_time` is set to `time + delay`, so staggered lines
  begin at the correct offset.
- Completed animations are automatically removed from the active list.
