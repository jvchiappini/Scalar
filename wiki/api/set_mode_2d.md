# set_mode_2d

**Syntax:** `Camera.set_mode_2d(width, height)`

**Description:**
Configures the camera in a pure 2D orthographic projection mode. The point `(0, 0)` is located at the exact center of the canvas. The unit scale is adjusted so that the visible area matches the specified width and height, independent of the window's aspect ratio.

**Arguments:**
- `width` (Number): Visible width in world units.
- `height` (Number): Visible height in world units.

**Example:**
```scalar
// Configure a 16:9 canvas with the origin at the center
Camera.set_mode_2d(16, 9)
```
