# set_fill

**Syntax:** `obj.set_fill(r, g, b, a)`

**Description:**
Defines the fill color for a 2D vector object. If this function is not called, the object may be rendered only with a stroke if one has been configured.

**Arguments:**
- `r, g, b, a` (Numbers): Color components (0.0 to 1.0).

**Example:**
```scalar
let r = Rect(4, 2)
r.set_fill(0.8, 0.2, 0.2, 1.0) // Red fill
```
