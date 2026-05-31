# Scalar Language Specification

This document is the entry point for the Scalar language grammar and built-in function reference.

Scalar is a declarative-execution language designed for visual mathematics and animation.

---

## Quick Index

| Topic | Description | File |
|-------|-------------|------|
| **Core Syntax** | Types, statements, kwargs, OO syntax, color constants | [syntax.md](syntax.md) |
| **Axes()** | Cartesian axes with grid, ticks, arrows — full kwarg reference | [axes.md](axes.md) |
| **Plot()** | Mathematical function plotting — full kwarg reference | [plot.md](plot.md) |
| **Shapes** | Line, Rect, Circle — constructors and style methods | [shapes.md](shapes.md) |
| **Project Settings** | Resolution, Background, SetFPS, MotionBlur | [project.md](project.md) |
| **Animation** | Animate, SetLineProgress, SetLineCap | [animation.md](animation.md) |

---

## Function Index (Quick Reference)

### Shapes
| Function | Returns | File |
|----------|---------|------|
| `Line(x1, y1, x2, y2 [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Rect(x, y, width, height [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Circle(x, y, radius [, ...])` | NodeId | [shapes.md](shapes.md) |

### Math Primitives
| Function | Returns | File |
|----------|---------|------|
| `Axes(x_min, x_max, y_min, y_max [, kwargs...])` | Number | [axes.md](axes.md) |
| `Plot("expr", x_min, x_max [, kwargs...])` | Number | [plot.md](plot.md) |

### Project
| Function | Description | File |
|----------|-------------|------|
| `Resolution(width, height)` | Set render dimensions | [project.md](project.md) |
| `Background(r, g, b [, a])` | Set clear color | [project.md](project.md) |
| `SetFPS(fps)` | Override frame rate | [project.md](project.md) |
| `MotionBlur(samples)` | Enable/disable motion blur | [project.md](project.md) |

### Animation
| Function | Description | File |
|----------|-------------|------|
| `SetLineProgress(node_id, progress)` | Show fraction of a line | [animation.md](animation.md) |
| `SetLineCap(node_id, cap)` | Change line cap style | [animation.md](animation.md) |
| `Animate(lines: [...], ...)` | Register draw animation | [animation.md](animation.md) |

---

## Complete Script Example

```scalar
// 4K 240fps animated demo with motion blur
Resolution(3840, 2160)
Background(0.05, 0.05, 0.1)
SetFPS(240)
MotionBlur(4)

// Animated axes with styled grid and ticks
Axes(-6, 6, -3, 3,
     grid: true,
     grid_width: 0.8,
     grid_alpha: 0.5,
     tick_step: 1.0,
     tick_direction: "outward",
     minor_ticks: 4,
     margin: 20,
     arrow_size: 1.5,
     animate: true,
     anim_duration: 1.5,
     anim_easing: "ease_out_cubic")

// Animated plots with different easings
Plot("sin(x)", -6, 6,
     samples: 300, thickness: 3,
     color: [1, 0.3, 0.3, 1],
     animate: true,
     anim_duration: 3.0,
     anim_easing: "ease_out_cubic")

Plot("cos(x)", -6, 6,
     samples: 300, thickness: 3,
     color: [0.3, 0.6, 1, 1],
     animate: true,
     anim_duration: 3.0,
     anim_easing: "ease_in_out_quart")

Plot("x^3/18", -4, 4,
     samples: 200, thickness: 3.5,
     color: [0.3, 1, 0.3, 1],
     animate: true,
     anim_duration: 3.5,
     anim_easing: "ease_out_bounce")
```

---

## See Also

- [Easing Function Reference](../api/easing.md) — all supported easing curves
- [Bindings Memory Map](../bindings/memory_map.md) — FFI layer between VM and renderer
- [Engine Architecture](../engine/architecture.md) — render pipeline and ECS lifecycle
