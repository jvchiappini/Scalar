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
| `anim_duration` | Number | `2.0` | Total time from first segment start to last segment end (seconds) |
| `anim_delay` | Number | `0.0` | Delay before the plot animation starts (seconds) |
| `anim_overlap` | Number | `0.5` | Overlap between consecutive segments: `0.0` = sequential, `1.0` = fully parallel |
| `anim_easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

### Animation Timing Model

The plot animation segments are arranged along the timeline using a configurable overlap model:

- **`anim_duration`** defines the total span from when the first segment starts drawing to when the last segment finishes.
- **`anim_overlap`** controls how much consecutive segments overlap:
  - `0.0` → **Sequential**: each segment fully completes before the next starts. Total time = `anim_duration`.
  - `0.5` → **Half overlap** (default): each segment overlaps 50% with the next for smooth continuous drawing.
  - `1.0` → **Parallel**: all segments draw simultaneously. Total time = `anim_duration`.
- **`anim_delay`** adds a leading pause before the first segment begins, useful for coordinating multiple plots.

Formula per segment:
```
segment_duration = anim_duration / (1 + (n-1) * (1 - overlap))
delay_between    = segment_duration * (1 - overlap)
delay[seg_i]     = anim_delay + i * delay_between
```

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

// Staggered plots with delay for sequential appearance
// First plot starts at t=0, second at t=1.5s, third at t=3s
Plot("sin(x)", -6, 6,
     animate: true,
     anim_duration: 1.5,
     anim_delay: 0.0,
     anim_overlap: 0.5)

Plot("cos(x)", -6, 6,
     animate: true,
     anim_duration: 1.5,
     anim_delay: 1.5,
     anim_overlap: 0.3)

Plot("x^2/12", -6, 6,
     animate: true,
     anim_duration: 2.0,
     anim_delay: 3.0,
     anim_overlap: 0.8)

// Fully sequential segments (no overlap)
Plot("tanh(x)", -6, 6,
     animate: true,
     anim_duration: 3.0,
     anim_overlap: 0.0)
```
