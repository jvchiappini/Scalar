# Scalar Grammar Specification

This document defines the formal grammar and syntax for the Scalar Language.

## Syntax Overview

Scalar is a declarative-execution language designed for visual mathematics and animation.

### Core Types
- **Number**: 64-bit floating point.
- **List**: Ordered collection of values: `[val1, val2, ...]`.
- **NodeId**: Reference to an entity in the rendering engine, returned by shape constructors.
- **String**: Text enclosed in double quotes: `"hello"`.
- **Color**: RGBA vector (4-component List or individual components).

### Statements
- **Variable Declaration**: `let x = 10`
- **For Loop**: `for i in 0..10 { ... }`
- **Method Call**: `object.method(args)`
- **Import**: `import "filename.scl"` (merges functions and variables with the current scope)

### Named Arguments (Kwargs)
Functions and methods accept keyword arguments after positional ones:
```
func(pos1, pos2, key1: value1, key2: value2)
```

---

## Built-in Functions

### Shapes
| Function | Description |
|----------|-------------|
| `Line(x1, y1, x2, y2 [, thickness [, r, g, b, a]] [, cap: "round"])` | 2D line segment |
| `Rect(x, y, width, height [, r, g, b, a])` | 2D filled rectangle |
| `Circle(x, y, radius [, r, g, b, a])` | 2D filled circle |

### Project
| Function | Description |
|----------|-------------|
| `Resolution(width, height)` | Set render target dimensions |
| `Background(r, g, b [, a])` | Set background clear color |
| `SetFPS(fps)` | Override output frame rate |
| `MotionBlur(samples)` | Enable/disable motion blur (0 = off, >0 = sub-samples) |

### Math Primitives
| Function | Description |
|----------|-------------|
| `Axes(x_min, x_max, y_min, y_max [, grid:, tick_step:, ...])` | Cartesian axes with grid, ticks, arrows |
| `Plot("expr", x_min, x_max [, samples:, thickness:, color:, cap:])` | Plot mathematical function `f(x)` |

### Animation
| Function | Description |
|----------|-------------|
| `SetLineProgress(node_id, progress)` | Shows fraction `[0,1]` of a line |
| `SetLineCap(node_id, cap)` | Changes line cap style (`"round"`, `"square"`, `"flat"`) |
| `Animate(lines: [id, ...], per_line: 1.0, staggered: true, easing: "ease_out_cubic")` | Registers a line-draw animation with easing |

`Animate()` kwargs:

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `lines` | `[NodeId]` | (required) | Lines to animate |
| `per_line` | Number | `1.0` | Duration per line (seconds) |
| `duration` | Number | — | Total duration (alternative to `per_line`) |
| `staggered` | Boolean | `true` | If `true`, lines animate sequentially |
| `easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

### Style Methods
| Method | Description |
|--------|-------------|
| `obj.set_fill(r, g, b, a)` | Sets fill color |
| `obj.set_stroke(r, g, b, a, thickness)` | Sets stroke color and thickness |
| `obj.set_z_index(z)` | Sets 2D painter's order |
| `obj.morph_to(target, duration:, easing:)` | Morphs geometry to another shape |

### Standard Colors
`WHITE`, `BLACK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`

---

## Object-Oriented Syntax
Scalar supports method injection. If `obj` is a `NodeId`, calling `obj.method(...)` is equivalent to a native call where `obj` is the first argument.

```scalar
let c = Circle(50)
c.set_fill(RED)
```

---

## Examples

```scalar
// Linea con puntas cuadradas animada
let l = Line(-200, 0, 200, 0, 10, cap: "square")
Animate(lines: [l], per_line: 2.0)

// Multiples lineas secuenciales
let a = Line(-300, 0, 300, 0, 20, 1, 0, 0)
let b = Line(0, -200, 0, 200, 20, 0, 1, 0)
Animate(lines: [a, b], per_line: 1.5, staggered: true)
```
