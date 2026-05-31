# Shape Functions

Primitive 2D shape constructors. All coordinates are in the Pure2D system:
- Origin (0, 0) = center of the screen
- 1 unit = 1 pixel exactly
- X-axis: positive = right, negative = left
- Y-axis: positive = up, negative = down

---

## Line

```scalar
// Minimal — white 1px line
Line(x1, y1, x2, y2)

// With thickness
Line(x1, y1, x2, y2, thickness)

// With thickness and color
Line(x1, y1, x2, y2, thickness, r, g, b, a)

// With cap style (kwargs)
Line(x1, y1, x2, y2, thickness, cap: "round")

// Point-list syntax
Line([x1, y1], [x2, y2], thickness, color)
```

**Returns:** NodeId

### Kwargs
| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `cap` | String | `"round"` | Line cap style: `"round"`, `"square"`, `"flat"` |

---

## Rect

```scalar
// White filled rectangle
Rect(x, y, width, height)

// With color
Rect(x, y, width, height, r, g, b, a)
```

Centered at (x, y). Returns NodeId.

---

## Circle

```scalar
// White filled circle
Circle(x, y, radius)

// With color
Circle(x, y, radius, r, g, b, a)
```

Centered at (x, y). Returns NodeId.

---

## Style Methods

Called on a NodeId returned by a shape constructor:

| Method | Description |
|--------|-------------|
| `obj.set_fill(r, g, b, a)` | Sets fill color |
| `obj.set_stroke(r, g, b, a, thickness)` | Sets stroke color and thickness |
| `obj.set_z_index(z)` | Sets 2D painter's order (higher = on top) |
| `obj.morph_to(target, duration:, easing:)` | Morphs geometry to another shape |

### Examples

```scalar
let c = Circle(50)
c.set_fill(RED)

let l = Line(-100, 0, 100, 0, 5)
l.set_stroke(1, 1, 1, 1, 3)
l.set_z_index(10)

let r = Rect(0, 0, 200, 100, YELLOW)
r.set_z_index(-5)
```
