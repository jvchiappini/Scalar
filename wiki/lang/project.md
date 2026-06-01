# Project Functions

Global settings that affect the entire render output.

---

## Resolution

```scalar
Resolution(width, height)
```

Sets the render target dimensions in pixels.

---

## Background

```scalar
Background(r, g, b)
Background(r, g, b, a)
```

Sets the background clear color. Values are in `[0, 1]` range.

---

## SetFPS

```scalar
SetFPS(fps)
```

Overrides the output frame rate. Used in conjunction with the CLI `--duration` flag to determine total frame count.

---

## MotionBlur

```scalar
MotionBlur(samples)
```

Enables or disables motion blur. When `samples > 0`, each output frame is an average of `samples` sub-frames rendered at `fps * samples` effective rate.

- `0` = motion blur disabled (default)
- `1` = single sub-sample (no blur, but uses accumulation pipeline)
- `2+` = motion blur with N sub-samples

Higher sample counts produce smoother motion blur at the cost of linearly increased render time.

---

## SetVisibility

```scalar
SetVisibility(node_id, visible)
```

Shows or hides a node. `visible` is a boolean (`true` or `false`).

### Example

```scalar
let shape = Circle(0, 0, 50)
SetVisibility(shape, false)  // hide
// ... later ...
SetVisibility(shape, true)   // show
```

---

## Example

```scalar
Resolution(3840, 2160)
Background(0.05, 0.05, 0.1)
SetFPS(240)
MotionBlur(4)
```
