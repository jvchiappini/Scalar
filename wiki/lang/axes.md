# Axes() — Full Keyword Reference

`Axes(x_min, x_max, y_min, y_max [, kwargs...])` draws cartesian axes with grid lines, tick marks, and arrowheads. Coordinates are mathematical — viewport scaling is automatic.

---

## Grid

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `grid` | Boolean | `false` | Show grid lines |
| `grid_color` | [r,g,b,a] | `[0.2, 0.2, 0.25, 1.0]` | Grid line color |
| `grid_width` | Number | `1.0` | Grid line thickness in pixels |
| `grid_alpha` | Number | `1.0` | Grid opacity multiplier (0.0 = invisible, 1.0 = full) |

---

## Ticks

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `tick_step` | Number | auto | Step between major tick marks (0 = auto-compute from range) |
| `tick_len` | Number | `6.0` | Tick mark length in pixels |
| `tick_width` | Number | `1.5` | Tick mark line thickness |
| `tick_direction` | String | `"both"` | Tick direction (see below) |
| `minor_ticks` | Number | `0` | Number of minor ticks between each pair of major ticks |

### tick_direction values

| Value | Behavior |
|-------|----------|
| `"both"` | Ticks are centered on the axis line (extends equally both ways) |
| `"outward"` | Ticks extend outward from the axis (away from first quadrant) |
| `"inward"` | Ticks extend inward from the axis (toward first quadrant) |
| `"none"` | No tick marks are drawn |

---

## Axis Lines

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `axis_color` | [r,g,b,a] | `[0.5, 0.5, 0.5, 1.0]` | Axis line color (both axes) |
| `x_axis_color` | [r,g,b,a] | `axis_color` | Override color for X-axis only |
| `y_axis_color` | [r,g,b,a] | `axis_color` | Override color for Y-axis only |
| `axis_width` | Number | `2.0` | Axis line thickness in pixels |
| `show_x` | Boolean | `true` | Show X-axis line, ticks, and arrow |
| `show_y` | Boolean | `true` | Show Y-axis line, ticks, and arrow |

---

## Arrowheads

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `arrows` | Boolean | `true` | Show arrowheads at positive axis ends |
| `arrow_size` | Number | `1.0` | Arrowhead size multiplier (>1 = larger, <1 = smaller) |

---

## Layout

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `aspect` | String | `"fit"` | Aspect policy: `"fit"` (maintains ratio) or `"stretch"` (fill viewport) |
| `origin` | String | `"zero"` | Where axes cross (see below) |
| `margin` | Number | `0.0` | Margin around the plot area in pixels |
| `x_padding` | Number | `0.0` | Fractional padding added to x-range (e.g. 0.05 = 5% on each side) |
| `y_padding` | Number | `0.0` | Fractional padding added to y-range |
| `z_index` | Number | `0` | Base z-order for all axes elements (higher = rendered on top) |

### origin values

| Value | Behavior |
|-------|----------|
| `"zero"` | Axes cross at math coordinate (0, 0). X-axis at y=0, Y-axis at x=0. |
| `"min"` | Axes placed at minimum bounds. X-axis at y=y_min, Y-axis at x=x_min. Useful for charts where data does not cross zero. |

---

## Animation

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `animate` | Boolean | `false` | Enable draw-in animation for all axes elements |
| `anim_duration` | Number | `1.5` | Animation duration in seconds |
| `anim_easing` | String | `"ease_out_cubic"` | Easing function name (see [Easing](../api/easing.md)) |

---

## Examples

```scalar
// Minimal axes
Axes(-10, 10, -5, 5)

// Full-featured axes with grid, styled ticks, and animation
Axes(-6, 6, -3, 3,
     grid: true,
     grid_width: 0.8,
     grid_alpha: 0.5,
     tick_step: 1.0,
     tick_direction: "outward",
     minor_ticks: 4,
     x_axis_color: [1, 0.5, 0.5, 1],
     y_axis_color: [0.5, 0.7, 1, 1],
     axis_width: 2.5,
     margin: 20,
     x_padding: 0.05,
     origin: "zero",
     arrows: true,
     arrow_size: 1.5,
     z_index: 5,
     animate: true,
     anim_duration: 2.0,
     anim_easing: "ease_out_quart")

// Axes with origin at minimum bounds (chart-style, bottom-left)
Axes(0, 10, 0, 5,
     grid: true,
     origin: "min",
     arrows: true)

// Axes with per-axis colors and inward ticks
Axes(-3, 3, -2, 2,
     tick_direction: "inward",
     tick_len: 8,
     x_axis_color: [1, 0.4, 0.4, 1],
     y_axis_color: [0.4, 0.7, 1, 1],
     grid: true,
     grid_alpha: 0.3)

// Dark subtle axes with only X-axis visible
Axes(-5, 5, -5, 5,
     show_y: false,
     axis_color: [0.3, 0.3, 0.3, 1],
     arrows: false,
     tick_direction: "none")
```
