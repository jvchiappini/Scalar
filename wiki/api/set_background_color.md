# set_background_color

**Syntax:** `set_background_color(r, g, b, a)` or `set_background_color(color_list)`

**Description:**
Sets the background color of the scene and disables the 3D environment (skybox). This is especially useful for 2D animations where a clean, solid background is required.

**Arguments:**
- `r`, `g`, `b`, `a` (Numbers): RGBA components from 0.0 to 1.0.
- `color_list` (List): A list containing [r, g, b, a]. You can use predefined constants like `BLACK`, `WHITE`, `BLUE`, etc.

**Example:**
```scalar
// Set background to a deep blue
set_background_color(0.05, 0.05, 0.2, 1.0)

// Use a predefined constant
set_background_color(BLACK)
```

> [!NOTE]
> Calling `Camera.set_mode_2d()` automatically calls `set_background_color(BLACK)` to ensure a clean slate for 2D work.
