# Plot

**Syntax:** `Plot("expression", x_min, x_max [, kwargs...])`

**Description:**
Evaluates and draws a mathematical function `f(x)` sampled over the range `[x_min, x_max]`, connecting samples with line segments. Uses the same mathematical coordinate system as `Axes()` — viewport scaling is automatic.

---

## Positional Arguments

| Argument | Type | Required | Description |
|----------|------|:--------:|-------------|
| `expression` | String | ✓ | Math expression in terms of `x` |
| `x_min` | Number | ✓ | Lower bound of the domain |
| `x_max` | Number | ✓ | Upper bound of the domain |

---

## Keyword Arguments

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `samples` | Number | `200` | Number of sample points (higher = smoother) |
| `thickness` | Number | `3.0` | Line thickness in pixels |
| `color` | [r,g,b,a] | `[1,1,1,1]` | Curve color |
| `cap` | String | `"round"` | Line cap style: `"round"`, `"square"`, `"flat"` |
| `animate` | Boolean | `false` | Enable draw-in animation |
| `anim_duration` | Number | `2.0` | Total time from first segment start to last segment end (s) |
| `anim_delay` | Number | `0.0` | Delay before plot animation starts (seconds) |
| `anim_overlap` | Number | `0.5` | Segment overlap: `0.0`=sequential, `1.0`=parallel |
| `anim_easing` | String | `"ease_out_cubic"` | Easing function (see [Easing](easing.md)) |

---

## Expression Syntax

The math evaluator supports:

### Operators
`+`, `-`, `*`, `/`, `^` (power, right-associative)

### Variable
`x`

### Constants
`pi`, `e`

### Trigonometric
`sin(x)`, `cos(x)`, `tan(x)`, `asin(x)`, `acos(x)`, `atan(x)`

### Other
`sqrt(x)`, `abs(x)`, `log(x)` (base 10), `ln(x)`, `exp(x)`

### Rounding
`floor(x)`, `ceil(x)`, `round(x)`, `sign(x)`

### Binary
`atan2(y, x)`, `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)`

---

## Full Reference

See [wiki/lang/plot.md](../lang/plot.md) for the complete documentation with examples.
