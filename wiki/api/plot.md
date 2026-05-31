# Plot

**Syntax:** `Plot("expression", x_min, x_max [, kwargs...])`

**Description:**
Evalúa y dibuja una función matemática `f(x)` muestreándola en el rango `[x_min, x_max]` y conectando los puntos con segmentos de línea.

Usa el mismo sistema de coordenadas matemáticas que `Axes()`: el escalado al viewport es automático.

---

## Arguments

| Argument | Type | Obligatorio | Description |
|----------|------|:-----------:|-------------|
| `expression` | String | ✓ | Expresión matemática en términos de `x` |
| `x_min` | Number | ✓ | Límite inferior del dominio |
| `x_max` | Number | ✓ | Límite superior del dominio |

### Keyword Arguments

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `samples` | Number | `200` | Número de puntos de muestreo (mayor = más suave) |
| `thickness` | Number | `3.0` | Grosor de la línea (píxeles) |
| `color` | `[r,g,b,a]` | `[1,1,1,1]` | Color de la curva |
| `cap` | String | `"round"` | Estilo de punta: `"round"`, `"square"`, `"flat"` |
| `animate` | Boolean | `false` | Habilita animación de dibujo progresivo (trazo) |
| `anim_duration` | Number | `2.0` | Duración total de la animación (segundos) |
| `anim_easing` | String | `"ease_out_cubic"` | Función de easing (ver [Easing](easing.md)) |

---

## Expression syntax

El evaluador matemático soporta:

| Categoría | Operadores/Funciones |
|-----------|----------------------|
| **Aritmética** | `+`, `-`, `*`, `/`, `^` (potencia, asociativo a derecha) |
| **Variable** | `x` |
| **Constantes** | `pi`, `e` |
| **Trigonométricas** | `sin(x)`, `cos(x)`, `tan(x)`, `asin(x)`, `acos(x)`, `atan(x)` |
| **Otras** | `sqrt(x)`, `abs(x)`, `log(x)` (base 10), `ln(x)`, `exp(x)` |
| **Redondeo** | `floor(x)`, `ceil(x)`, `round(x)`, `sign(x)` |
| **Binarias** | `atan2(y, x)`, `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)` |

---

## Example

```scalar
Resolution(800, 600)
Background(0.05, 0.05, 0.1)

Axes(-6, 6, -4, 4, grid: true, tick_step: 1.0)

// Seno en rojo
Plot("sin(x)", -6, 6, samples: 300, thickness: 3,
     color: [1.0, 0.3, 0.3, 1.0])

// Coseno en azul
Plot("cos(x)", -6, 6, samples: 300, thickness: 3,
     color: [0.3, 0.6, 1.0, 1.0])

// Polinomio cúbico
Plot("x^3/12 - x", -6, 6, thickness: 2.5,
     color: [0.3, 1.0, 0.3, 1.0])

// Parábola con puntas cuadradas
Plot("x^2/4", -6, 6, thickness: 4,
     color: [1.0, 1.0, 0.3, 1.0],
     cap: "square")

// Curva animada — el trazo se dibuja progresivamente
Plot("sin(x)", -6, 6, samples: 300, thickness: 3,
     color: [1.0, 0.3, 0.3, 1.0],
     animate: true,
     anim_duration: 3.0,
     anim_easing: "ease_out_cubic")
```

---

## Notes

- Los puntos donde `f(x)` es infinito o `NaN` se omiten automáticamente (gestión de discontinuidades).
- Cada `Plot()` crea `samples - 1` entidades de línea en el ECS.
- Para funciones con picos pronunciados, aumenta `samples` a 500–1000.
- Con `animate: true`, cada segmento se anima con un stagger automático — la curva se dibuja de izquierda a derecha.
- La duración `anim_duration` controla el tiempo total; los segmentos usan `duration / samples * 1.5` cada uno con ligero solapamiento para suavidad.
