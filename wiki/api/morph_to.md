# morph_to

**Syntax:** `source.morph_to(target, duration: seconds, easing: "type")`

**Description:**  
Animates the geometry of `source` towards the geometry of `target` over `duration` seconds.
The engine performs a vertex-level interpolation between the two `PathData` command lists,
applying the chosen easing curve on every frame.

If `source` and `target` have a different number of path commands the interpolation is
truncated to the shorter sequence. For predictable results, use paired primitives
(e.g. `Rect` ↔ `Circle`) which are both tessellated with 4 cubic Bézier segments.

**The target shape is automatically hidden.**  
When `morph_to` is called, the engine sets `target.visible = false` internally.
You never need to manually call `target.hide()` — the mold is invisible from the
start and only the interpolation result is rendered.

---

## Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `target` | Shape Object | Object whose final geometry is the animation goal |
| `duration` | Number (keyword) | Animation length in seconds |
| `easing` | String (keyword) | Easing curve — see table below |

### Easing values

| String | Curve |
|--------|-------|
| `"linear"` | Constant rate |
| `"ease_in_out"` / `"easeInOut"` / `"easeInOutCubic"` | Smooth start and end (cubic) |

---

## Example

```scalar
Camera.set_mode_2d(16.0, 9.0)

let cuadrado = Rect(0, 0, 2, 2)
cuadrado.set_fill(BLUE)

// The circle is the mold — opacity(0) OR morph_to auto-hide both work.
let circulo = Circle(0, 0, 1.5)
circulo.set_opacity(0)   // optional: morph_to hides it anyway

cuadrado.morph_to(circulo, duration: 3.0, easing: "ease_in_out")
```

---

## Constraints

- `morph_to` is **one-shot**: the animation plays once from the current playhead
  position to `current_time + duration`.
- Both objects must be 2D `Path` entities (created via `Circle`, `Rect`, `Path`, etc.).
  3D meshes are not supported.
- For smooth morphing, ensure both shapes have the **same number of path commands**.
  `Rect` and `Circle` each use 4 cubic Bézier segments + 1 `Close` command by
  convention, making them directly compatible.

---

## Auto-hide behaviour (v15)

Prior to v15, calling `morph_to` would leave the target mold visible, causing it to
render as a stray shape (e.g. a grey square) on top of the morphing animation.

Since v15, `morph_to` unconditionally calls `Renderer::set_visible(target, false)`
immediately when the animation job is registered — regardless of whether the target
already has `opacity(0)`. This eliminates the visual glitch without requiring any
change in the script.
