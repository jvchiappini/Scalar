# Plot() — Keyword Reference

`Plot("expression", x_min, x_max [, kwargs...])` evaluates and draws a mathematical function `f(x)` sampled over the interval `[x_min, x_max]`.

---

## Full Kwarg Reference

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `samples` | Number | `200` | Number of sample points (higher = smoother curves) |
| `thickness` | Number | `3.0` | Line thickness in pixels |
| `color` | [r,g,b,a] | `[1, 1, 1, 1]` | Line color |
| `cap` | String | `"round"` | Line cap style: `"round"`, `"square"`, `"flat"` |
| `animate` | Boolean | `false` | Enable draw-in animation (segments stagger left-to-right) |
| `anim_duration` | Number | `2.0` | Animation duration in seconds |
| `anim_easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

---

## Expression Syntax

The expression string supports standard math operators and functions:

### Operators
`+`, `-`, `*`, `/`, `^` (power), `()` for grouping

### Supported Functions
- `sin(x)`, `cos(x)`, `tan(x)`
- `asin(x)`, `acos(x)`, `atan(x)`
- `sinh(x)`, `cosh(x)`, `tanh(x)`
- `sqrt(x)`, `abs(x)`, `log(x)`, `ln(x)`
- `exp(x)`, `ceil(x)`, `floor(x)`, `round(x)`

### Supported Constants
- `pi`, `e`

### Examples

```scalar
Plot("sin(x)", -6, 6)
Plot("x^2", -4, 4)
Plot("1/x", 0.1, 5)
Plot("sqrt(abs(x))", -9, 9)
Plot("e^(-x^2/2)", -4, 4)
Plot("sin(x)*cos(x)", -6, 6)
Plot("x^3 - 2*x + 1", -3, 3)
```

---

## Examples

```scalar
// Basic plot
Plot("sin(x)", -6, 6)

// Styled plot with animation
Plot("cos(x)", -6, 6,
     samples: 300,
     thickness: 3,
     color: [0.3, 0.6, 1, 1],
     cap: "round",
     animate: true,
     anim_duration: 3.0,
     anim_easing: "ease_in_out_quart")

// Thick polynomial with bounce easing
Plot("x^3/18", -4, 4,
     samples: 200,
     thickness: 4.5,
     color: [0.3, 1, 0.3, 1],
     animate: true,
     anim_duration: 3.5,
     anim_easing: "ease_out_bounce")
```
