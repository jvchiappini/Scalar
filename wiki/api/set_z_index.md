# set_z_index

**Syntax:** `obj.set_z_index(z)`

**Description:**
Sets the layer order for the Painter's Algorithm on the 2D canvas. Objects with a higher `z_index` are drawn on top of those with a lower value. This prevents depth conflicts (Z-fighting) in orthographic mode.

**Arguments:**
- `z` (Number): Integer value defining relative depth.

**Example:**
```scalar
let back = Rect(10, 10)
let front = Circle(2)

back.set_z_index(0)
front.set_z_index(10) // Always on top of the rectangle
```
