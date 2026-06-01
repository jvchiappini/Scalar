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
| **Text & Import** | SVGImport, FontImport, Text — file loading and vector text | [text.md](text.md) |
| **Project Settings** | Resolution, Background, SetFPS, MotionBlur | [project.md](project.md) |
| **Animation** | Animate, FadeIn, FadeOut, Grow, Shrink, MoveTo, SetLineProgress, SetLineCap, WriteText | [animation.md](animation.md) |
| **Text & Import** | SVGImport, FontImport, Text | [text.md](text.md) |

---

## Function Index (Quick Reference)

### Shapes
| Function | Returns | File |
|----------|---------|------|
| `Line(x1, y1, x2, y2 [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Rect(x, y, width, height [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Circle(x, y, radius [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Triangle(x, y, size [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Star(x, y, outer_r, inner_r, points [, ...])` | NodeId | [shapes.md](shapes.md) |
| `RegularPolygon(x, y, radius, sides [, ...])` | NodeId | [shapes.md](shapes.md) |
| `Polygon([[x,y], ...] [, ...])` | NodeId | [shapes.md](shapes.md) |
| `SVG("path" [, ...])` | NodeId | [shapes.md](shapes.md) |

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
| Function | Returns | Description | File |
|----------|---------|-------------|------|
| `SetLineProgress(node_id, progress)` | Number | Show fraction of a line | [animation.md](animation.md) |
| `SetLineCap(node_id, cap)` | Number | Change line cap style | [animation.md](animation.md) |
| `Animate(lines: [...], ...)` | Number | Register line-draw animation | [animation.md](animation.md) |
| `FadeIn(node_id, ...)` | Number | Fade node from opacity 0→1 | [animation.md](animation.md) |
| `FadeOut(node_id, ...)` | Number | Fade node from opacity 1→0 | [animation.md](animation.md) |
| `Grow(node_id, ...)` | Number | Scale node from 0→1   | [animation.md](animation.md) |
| `Shrink(node_id, ...)` | Number | Scale node from 1→0   | [animation.md](animation.md) |
| `MoveTo(node_id, x, y, ...)` | Number | Slide node to position (x,y) | [animation.md](animation.md) |
| `DrawThenFill(node_id, ...)` | Number | Two-phase scale-up + fill-fade reveal | [animation.md](animation.md) |
| `WriteText(str, x, y, ...)` | `[NodeId]` | Character-by-character text fade-in reveal | [animation.md](animation.md) |
| `RevealText(str, x, y, ...)` | `[NodeId]` | Character-by-character draw-then-fill text reveal | [animation.md](animation.md) |

### Text & File Import
| Function | Returns | Description | File |
|----------|---------|-------------|------|
| `SVGImport(path)` | `List[NodeId]` | Load SVG file, render all `<path>` elements | [text.md](text.md) |
| `FontImport(path)` | `Number` | Load TrueType/OpenType font for text | [text.md](text.md) |
| `Text(str, x, y [, kwargs...])` | `[NodeId]` | Render text as vector paths with loaded font (one `NodeId` per glyph) | [text.md](text.md) |
| `Tex(expr, x, y [, kwargs...])` | `[NodeId]` | Render LaTeX math expression as vector paths (returns one `NodeId` per glyph/piece) | [text.md](text.md) |

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
