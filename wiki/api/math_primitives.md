# Arrow

**Syntax:** `Arrow(x1, y1, x2, y2)`

**Description:**
Creates a vector object representing an arrow from point `(x1, y1)` to `(x2, y2)`. Includes a proportionally scaled triangular tip.

**Arguments:**
- `x1, y1` (Numbers): Origin point.
- `x2, y2` (Numbers): Destination point (where the tip is located).

**Example:**
```scalar
let r = Arrow(0, 0, 5, 2)
r.set_color(1, 1, 0, 1)
```
---
# Axes

**Syntax:** `Axes(x_min, x_max, y_min, y_max)`

**Description:**
Draws a basic coordinate axis system with arrows at the ends.

**Arguments:**
- `x_min, x_max` (Numbers): X-axis range.
- `y_min, y_max` (Numbers): Y-axis range.

---
# Path

**Syntax:** `Path([[x,y], ...], thickness)`

**Description:**
Creates a polygonal path based on a list of points.

**Arguments:**
- `points` (List of Lists): Vertex coordinates.
- `thickness` (Number): Line thickness.
