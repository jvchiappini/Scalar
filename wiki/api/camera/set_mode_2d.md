# set_mode_2d

**Syntax:** `Camera.set_mode_2d(enabled)`

**Description:**
Toggles the camera between a 3D perspective projection and a 2D orthographic projection. 

When enabled, the point `(0, 0)` is fixed at the exact center of the screen, and the camera controls (orbit/zoom) are locked to prevent accidental shifts. The system automatically calculates the visible area based on project resolution and aspect ratio, defaulting to a logical height of **8.0 units** (industry standard for mathematical animations).

**Arguments:**
- `enabled` (Boolean): Pass `true` to activate 2D mode, `false` to return to 3D.

**Example:**
```scalar
// Set project resolution
set_resolution(1920, 1080)

// Enable 2D mode with fixed origin
Camera.set_mode_2d(true)

// A circle of radius 1 will occupy 1/4 of total screen height (8 units)
let c = Circle(0, 0, 1.0)
```

> [!TIP]
> `set_mode_2d(true)` automatically sets the background to solid black (`set_background_color(BLACK)`) and disables all external camera controllers to ensure the scene is perfectly centered and optimized for 2D graphics.
