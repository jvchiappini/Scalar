# set_stroke

**Syntax:** `obj.set_stroke(r, g, b, a, thickness)`

**Description:**
Defines the outline style (stroke) for a 2D vector object. It uses the Lyon algorithm with rounded joints and caps (`LineCap::Round`, `LineJoin::Round`) for a professional finish.

**Arguments:**
- `r, g, b, a` (Numbers): Color components (0.0 to 1.0).
- `thickness` (Number): Line thickness in world units.

**Example:**
```scalar
let c = Circle(2)
c.set_stroke(1, 1, 1, 1, 0.1) // White outline 0.1 thick
c.set_fill(0, 0, 1, 1)        // Blue fill
```
