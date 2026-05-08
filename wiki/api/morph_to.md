# morph_to

**Syntax:** `obj.morph_to(target, duration, easing)`

**Description:**
Animates an object's geometry towards the configuration of the `target` object. It performs a linear interpolation between the vector path commands. If the source and target objects have a different number of points, it will be truncated to the smaller one.

**Arguments:**
- `target` (Shape Object): The object that defines the final shape.
- `duration` (Number): Metamorphosis duration in seconds.
- `easing` (String): Easing type (e.g., `"easeInOut"`, `"linear"`).

**Example:**
```scalar
let c = Circle(2)
let r = Rect(4, 4)
r.set_visible(0) // The target does not need to be visible

c.morph_to(r, 2.0, "easeInOut")
```
