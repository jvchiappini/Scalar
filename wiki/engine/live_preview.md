# Live Preview & Hot-Reloading

`Live Preview` mode enables an interactive development experience where code changes are instantly reflected in the rendering window.

## How to use Preview mode

To start the CLI in interactive mode, use the `--preview` flag:

```bash
scalar_cli --input main.scl --preview
```

### Features
- **Hot-Reloading**: When saving the `.scl` file, the engine clears the scene, re-parses the script, and restarts execution without closing the window.
- **Robustness**: If there is a syntax or execution error during reload, the engine will display the error in the console using `ariadne` but will keep the last valid scene on screen.
- **Automatic Loading of std.scl**: The engine automatically searches for and loads the `std.scl` file in the working directory to provide constants (like `PI`) and convenience functions.

## Module System (Imports)

You can split your code into multiple files using the `import` statement:

```scalar
import "lib/materials.scl"
import "geometry.scl"

let ball = Sphere(10);
apply_metallic_material(ball);
```

Imported files are executed in the same `Environment` as the main file, allowing for the sharing of global variables and functions.

## Global Camera Control

The camera is available as a global `Camera` object:

- `Camera.set_position(x, y, z)`
- `Camera.look_at(x, y, z)`
- `Camera.set_rotation(p, y, r)` (Pitch, Yaw, Roll)
