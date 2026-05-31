# Line Cap Styles

**Syntax:**
- `Line(..., cap: "round")` — kwarg al crear la línea
- `SetLineCap(node_id, "square")` — cambiar después

**Description:**
Controls the shape of the endpoints of thick 2D lines. Three styles are available:

| Style    | Value | Appearance                                                |
|----------|-------|-----------------------------------------------------------|
| `"flat"` | `0`   | Ends exactly at the endpoints, straight cut (default for `Rect`, `Path`) |
| `"round"`| `1`   | Semicircular caps that extend to the endpoints (default for `Line()`)    |
| `"square"`| `2`  | Straight caps that extend `width/2` beyond the endpoints                |

---

## Visual difference

Given a line from `(0, 0)` to `(100, 0)` with `thickness = 20`:

- **Flat**:  A 100×20 rectangle from `(0, -10)` to `(100, 10)`.
- **Round**: A 120×20 pill with 10px-radius semicircles at both ends,
             centred on `(50, 0)`. The round caps reach exactly to `(0, 0)`
             and `(100, 0)`.
- **Square**: A 120×20 rectangle from `(-10, -10)` to `(110, 10)`.
              The extra 10px on each side extends width/2 past the endpoints.

---

## Example

```scalar
let round_l = Line(-200, 50, 200, 50, 20, 1, 0, 0, cap: "round")   // roja, puntas redondas
let square_l = Line(-200, 0, 200, 0, 20, 0, 1, 0, cap: "square")   // verde, puntas cuadradas
let flat_l = Line(-200, -50, 200, -50, 20, 0, 0, 1, cap: "flat")   // azul, puntas planas
```

---

## Implementation

The cap style is stored as `Element.line_cap_style: u8` (0=flat, 1=round, 2=square).
During `sync_ecs_to_shape_batcher`, the renderer calls `ShapeInstance::line_with_cap()`
with the stored style, which adjusts the SDF geometry length and corner radius:

- **Flat**:  `geom_length = length`, `corner_radius = 0`
- **Round**: `geom_length = length + width`, `corner_radius = width / 2`
- **Square**: `geom_length = length + width`, `corner_radius = 0`
