# Shape Functions

Primitive 2D shape constructors. All coordinates use the Pure2D system:
- Origin (0, 0) = center of the screen
- 1 unit = 1 pixel exactly
- X-axis: positive = right, negative = left
- Y-axis: positive = up, negative = down

---

## Unified Kwarg Reference

All shapes (except `Line`) accept these keyword arguments for full configuration:

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `fill` | [r,g,b,a] | `[1, 1, 1, 1]` | Fill color |
| `fill_color` | [r,g,b,a] | `[1, 1, 1, 1]` | Alias for `fill` |
| `stroke` | [r,g,b,a] | — | Stroke outline color (no stroke if omitted) |
| `stroke_width` | Number | `2.0` | Stroke thickness in pixels |
| `opacity` | Number | `1.0` | Global opacity (0.0 = invisible, 1.0 = opaque) |
| `z_index` | Number | `0` | Z-order (higher values render on top) |
| `rotation` | Number | `0` | Rotation in degrees (counter-clockwise) |
| `visible` | Boolean | `true` | Visibility |
| `cap` | String | `"round"` | Line cap for stroke: `"round"`, `"square"`, `"flat"` |

### Examples

```scalar
// Circle with orange fill and white stroke
Circle(0, 0, 50,
    fill: [1, 0.5, 0, 1],
    stroke: [1, 1, 1, 1],
    stroke_width: 3,
    z_index: 5)

// Rotated rectangle with opacity
Rect(100, 0, 80, 40,
    fill: RED,
    opacity: 0.7,
    rotation: 45)
```

---

## Line

```scalar
// Minimal — white 1px line
Line(x1, y1, x2, y2)

// With thickness and color (positional)
Line(x1, y1, x2, y2, thickness, r, g, b, a)

// With kwargs
Line(x1, y1, x2, y2, stroke: RED, stroke_width: 3, cap: "round")

// Point-list syntax
Line([x1, y1], [x2, y2], thickness, color)
```

**Returns:** NodeId

### Line-specific Kwargs

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `stroke` | [r,g,b,a] | `[1,1,1,1]` | Line color |
| `stroke_width` | Number | `1.0` | Line thickness |
| `cap` | String | `"round"` | Cap style: `"round"`, `"square"`, `"flat"` |
| `opacity` | Number | `1.0` | Opacity |
| `z_index` | Number | `0` | Z-order |
| `visible` | Boolean | `true` | Visibility |

---

## Rect

```scalar
Rect(x, y, width, height [, r, g, b, a] [, kwargs...])
```

Centered at `(x, y)`. Returns NodeId.

---

## Circle

```scalar
Circle(x, y, radius [, r, g, b, a] [, kwargs...])
```

Centered at `(x, y)`. Returns NodeId.

---

## Triangle

```scalar
Triangle(x, y, size [, kwargs...])
```

Equilateral triangle centered at `(x, y)`, pointing upward. `size` is the side length.

### Examples

```scalar
Triangle(0, 0, 60, fill: CYAN, stroke: BLUE, stroke_width: 2)
Triangle(-100, 50, 40, fill: RED, rotation: 180)
```

---

## Star

```scalar
Star(x, y, outer_radius, inner_radius, points [, kwargs...])
```

Multi-pointed star centered at `(x, y)`.

| Arg | Description |
|-----|-------------|
| `outer_radius` | Distance from center to outer points |
| `inner_radius` | Distance from center to inner points |
| `points` | Number of star points (≥ 3) |

### Examples

```scalar
// 5-pointed star
Star(0, 0, 80, 35, 5, fill: YELLOW, stroke: ORANGE)

// 8-pointed star, rotated
Star(150, 0, 60, 25, 8, fill: MAGENTA, rotation: 22.5)
```

---

## RegularPolygon

```scalar
RegularPolygon(x, y, radius, sides [, kwargs...])
```

Regular polygon centered at `(x, y)`.

| Arg | Description |
|-----|-------------|
| `radius` | Distance from center to vertices |
| `sides` | Number of sides (≥ 3) |

### Examples

```scalar
// Hexagon
RegularPolygon(0, 0, 50, 6, fill: GREEN, stroke: WHITE)

// Pentagon, rotated
RegularPolygon(100, 0, 40, 5, fill: [0.5, 0.2, 0.8, 1], rotation: 90)
```

---

## Polygon

```scalar
Polygon([[x1, y1], [x2, y2], [x3, y3], ...] [, kwargs...])
```

Arbitrary polygon from a list of `[x, y]` points. Minimum 3 points. The shape is centered at the centroid of the points.

### Examples

```scalar
// Custom triangle
Polygon([[0, -50], [-40, 30], [40, 30]],
    fill: RED, stroke: WHITE, stroke_width: 2)

// Irregular polygon
Polygon([[-30, -20], [20, -40], [50, 10], [0, 40], [-40, 10]],
    fill: [0.3, 0.6, 1, 0.8], stroke: [1, 1, 1, 0.5])
```

---

## SVG (SVG Path Rendering)

```scalar
SVG("path_data" [, x:, y:, scale:, kwargs...])
```

Renders an SVG path string into 2D geometry.

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `x` | Number | `0` | X offset (centered) |
| `y` | Number | `0` | Y offset (centered) |
| `scale` | Number | `1.0` | Uniform scale factor |

### Supported SVG Commands

| Command | Name | Description |
|---------|------|-------------|
| `M x y` | MoveTo (absolute) | Move pen to (x, y) |
| `m dx dy` | MoveTo (relative) | Move pen by (dx, dy) |
| `L x y` | LineTo (absolute) | Line to (x, y) |
| `l dx dy` | LineTo (relative) | Line by (dx, dy) |
| `H x` | Horizontal line | Horizontal line to x |
| `h dx` | Horizontal line (rel) | Horizontal line by dx |
| `V y` | Vertical line | Vertical line to y |
| `v dy` | Vertical line (rel) | Vertical line by dy |
| `C x1 y1 x2 y2 x y` | Cubic bezier (abs) | Cubic curve to (x,y) with control points |
| `c dx1 dy1 dx2 dy2 dx dy` | Cubic bezier (rel) | Relative cubic curve |
| `S x2 y2 x y` | Smooth cubic (abs) | Smooth cubic with reflected control point |
| `s dx2 dy2 dx dy` | Smooth cubic (rel) | Relative smooth cubic |
| `Q x1 y1 x y` | Quadratic bezier (abs) | Quadratic curve (converted to cubic) |
| `q dx1 dy1 dx dy` | Quadratic bezier (rel) | Relative quadratic |
| `T x y` | Smooth quadratic (abs) | Smooth quadratic with reflected control |
| `t dx dy` | Smooth quadratic (rel) | Relative smooth quadratic |
| `Z` / `z` | ClosePath | Close the current sub-path |

### Examples

```scalar
// Heart shape
SVG("M 0 15 C -20 -10, -40 10, 0 40 C 40 10, 20 -10, 0 15 Z",
    x: 0, y: 0, scale: 1.5,
    fill: [1, 0.2, 0.2, 1],
    stroke: [1, 1, 1, 0.8])

// Diamond
SVG("M 0 -30 L 30 0 L 0 30 L -30 0 Z",
    fill: CYAN, stroke: WHITE, stroke_width: 2)

// Spiral approximation
SVG("M 0 0 C 10 0, 10 10, 0 10 C -10 10, -10 -10, 0 -10 C 15 -10, 20 0, 20 15",
    fill: none, stroke: MAGENTA, stroke_width: 3)
```

---

## Style Methods

Called on any NodeId returned by a shape constructor:

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

let t = Triangle(0, 0, 60)
t.set_stroke(1, 1, 1, 1, 3)
t.set_z_index(10)

let r = Rect(0, 0, 200, 100, fill: YELLOW)
r.set_z_index(-5)
```
