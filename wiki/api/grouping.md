# Grouping & Style Propagation

Certain primitives in Scalar return a **Group** object instead of a single shape.
A Group is a logical collection of child shapes that share a common style interface.

## Primitives that return Groups

| Primitive | Children |
|-----------|----------|
| `Axes(x1, x2, y1, y2)` | X-axis line + Y-axis line |

---

## Style propagation (recursive)

When you call a style method on a Group, it is automatically applied to **every child** in the group.
You do not need to loop manually over the children.

```scalar
let plano = Axes(-5.0, 5.0, -3.0, 3.0)

// This sets the stroke on BOTH the X and Y axis lines:
plano.set_stroke(WHITE, 0.05)
```

### Available methods on Group objects

| Method | Effect |
|--------|--------|
| `set_stroke(color, thickness)` | Stroke color + width for all children |
| `set_fill(color)` | Fill color for all children |
| `set_opacity(value)` | Opacity (0.0–1.0) for all children |
| `set_z_index(int)` | Render order for all children |
| `hide()` | Makes all children invisible |
| `show()` | Makes all children visible |
| `morph_to(target, ...)` | Morphs the **first** child towards `target` |

---

## Visibility & hide / show

Both Group and individual shape objects expose `hide()` and `show()`:

```scalar
let c = Circle(0, 0, 1.5)
c.hide()          // invisible (but still in the ECS)
c.show()          // visible again
```

> [!NOTE]
> `morph_to` internally calls `hide()` on the **target** shape so that the mold
> never appears on screen. You do not need to hide it manually.

---

## Implementation detail

Style propagation is handled entirely in `scalar_bridge/src/bindings/shapes.rs`.
The Group stores a `Vec<NodeId>` and every style closure iterates over them,
forwarding the call to `Renderer::set_stroke` / `set_fill` / `set_opacity` / etc.

No ECS hierarchy or parent–child component is required — this is a pure
scripting-layer abstraction.
