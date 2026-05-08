# Standard Colors

Scalar v14 includes a set of predefined color constants for quick styling. These are 4-component RGBA lists.

| Constant | Color | RGBA Value |
| :--- | :--- | :--- |
| `WHITE` | White | `[1.0, 1.0, 1.0, 1.0]` |
| `BLACK` | Black | `[0.0, 0.0, 0.0, 1.0]` |
| `RED`   | Red   | `[1.0, 0.0, 0.0, 1.0]` |
| `GREEN` | Green | `[0.0, 1.0, 0.0, 1.0]` |
| `BLUE`  | Blue  | `[0.0, 0.0, 1.0, 1.0]` |
| `YELLOW`| Yellow| `[1.0, 1.0, 0.0, 1.0]` |
| `CYAN`  | Cyan  | `[0.0, 1.0, 1.0, 1.0]` |
| `MAGENTA`| Magenta| `[1.0, 0.0, 1.0, 1.0]` |

**Usage:**
```scalar
let c = Circle(2)
c.set_fill(BLUE)
c.set_stroke(WHITE, 0.05)
```
