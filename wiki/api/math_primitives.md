# Axes

**Syntax:** `Axes(x_min, x_max, y_min, y_max)`

**Description:**
Generates a coordinate system of axes. Returns a **Group** object that allows bulk styling of the axis lines.

**Example:**
```scalar
let plano = Axes(-5, 5, -3, 3)
plano.set_stroke(WHITE, 0.02)
```

---

# Plot

**Syntax:** `Plot(expression, x_min, x_max)`

**Description:**
Generates a path representing a mathematical function. 
*Note: In v14, expressions should be passed as strings or implicitly as x*x in Plot.*

**Example:**
```scalar
let parabola = Plot("x * x", -2.0, 2.0)
parabola.set_stroke(YELLOW, 0.05)
```
