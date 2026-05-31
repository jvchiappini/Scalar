# Axes

**Syntax:** `Axes(x_min, x_max, y_min, y_max [, kwargs...])`

**Description:**
Draws a cartesian coordinate system centered on the screen, with automatic scaling from mathematical coordinates to pixels.

The origin `(0, 0)` is at the center of the viewport. The Y-axis points upward. Scaling maintains aspect ratio by default.

---

## Positional Arguments

| Argument | Type | Required | Description |
|----------|------|:--------:|-------------|
| `x_min`  | Number | ✓ | Left bound of the X-axis |
| `x_max`  | Number | ✓ | Right bound of the X-axis |
| `y_min`  | Number | ✓ | Bottom bound of the Y-axis |
| `y_max`  | Number | ✓ | Top bound of the Y-axis |

---

## Full Keyword Argument Reference

### Grid

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `grid` | Boolean | `false` | Show grid lines |
| `grid_color` | [r,g,b,a] | `[0.2, 0.2, 0.25, 1.0]` | Grid line color |
| `grid_width` | Number | `1.0` | Grid line thickness in pixels |
| `grid_alpha` | Number | `1.0` | Grid opacity multiplier (0.0–1.0) |

### Ticks

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `tick_step` | Number | auto | Step between major tick marks (0 = auto-compute) |
| `tick_len` | Number | `6.0` | Tick mark length in pixels |
| `tick_width` | Number | `1.5` | Tick mark line thickness |
| `tick_direction` | String | `"both"` | `"both"`, `"outward"`, `"inward"`, `"none"` |
| `minor_ticks` | Number | `0` | Number of minor ticks between major ticks |

### Axis Lines

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `axis_color` | [r,g,b,a] | `[0.5, 0.5, 0.5, 1.0]` | Axis line color (both axes) |
| `x_axis_color` | [r,g,b,a] | `axis_color` | Override color for X-axis only |
| `y_axis_color` | [r,g,b,a] | `axis_color` | Override color for Y-axis only |
| `axis_width` | Number | `2.0` | Axis line thickness in pixels |
| `show_x` | Boolean | `true` | Show X-axis line, ticks, and arrow |
| `show_y` | Boolean | `true` | Show Y-axis line, ticks, and arrow |

### Arrowheads

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `arrows` | Boolean | `true` | Show arrowheads at positive axis ends |
| `arrow_size` | Number | `1.0` | Arrowhead size multiplier |

### Layout

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `aspect` | String | `"fit"` | `"fit"` (maintains ratio) or `"stretch"` (fills viewport) |
| `origin` | String | `"zero"` | Where axes cross: `"zero"` (at 0,0) or `"min"` (at x_min,y_min) |
| `margin` | Number | `0.0` | Margin around the plot area in pixels |
| `x_padding` | Number | `0.0` | Fractional x-range padding (e.g. 0.05 = 5%) |
| `y_padding` | Number | `0.0` | Fractional y-range padding |
| `z_index` | Number | `0` | Base z-order for all axes elements |

### Animation

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `animate` | Boolean | `false` | Enable draw-in animation |
| `anim_duration` | Number | `1.5` | Animation duration in seconds |
| `anim_easing` | String | `"ease_out_cubic"` | Easing function (see [Easing](easing.md)) |

---

## Full Reference

See [wiki/lang/axes.md](../lang/axes.md) for the complete documentation with examples.
