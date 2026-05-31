# Axes

**Syntax:** `Axes(x_min, x_max, y_min, y_max [, kwargs...])`

**Description:**
Dibuja un sistema de coordenadas cartesianas centrado en la pantalla, con escalado automático desde coordenadas matemáticas a píxeles.

El origen `(0, 0)` se sitúa en el centro de la ventana. El eje Y positivo apunta hacia arriba. El escalado mantiene la relación de aspecto por defecto.

---

## Arguments

| Argument | Type | Obligatorio | Description |
|----------|------|:-----------:|-------------|
| `x_min`  | Number | ✓ | Extremo izquierdo del eje X |
| `x_max`  | Number | ✓ | Extremo derecho del eje X |
| `y_min`  | Number | ✓ | Extremo inferior del eje Y |
| `y_max`  | Number | ✓ | Extremo superior del eje Y |

### Keyword Arguments

| Kwarg | Type | Default | Description |
|-------|------|---------|-------------|
| `grid` | Boolean | `false` | Muestra líneas de cuadrícula |
| `tick_step` | Number | automático | Distancia entre marcas de graduación |
| `axis_color` | `[r,g,b,a]` | `[0.5,0.5,0.5,1]` | Color de los ejes y ticks |
| `grid_color` | `[r,g,b,a]` | `[0.2,0.2,0.25,1]` | Color de la cuadrícula |
| `axis_width` | Number | `2.0` | Grosor de los ejes (píxeles) |
| `tick_len` | Number | `6.0` | Longitud de las marcas (píxeles) |
| `arrows` | Boolean | `true` | Muestra flechas en extremos positivos |
| `aspect` | String | `"fit"` | `"fit"` = mantiene proporción, `"stretch"` = llena la pantalla |
| `animate` | Boolean | `false` | Habilita animación de dibujo progresivo |
| `anim_duration` | Number | `1.5` | Duración total de la animación (segundos) |
| `anim_easing` | String | `"ease_out_cubic"` | Función de easing para la animación (ver [Easing](easing.md)) |

---

## Escalado automático

Las coordenadas son **matemáticas**. El engine calcula automáticamente la escala:

- Con `aspect: "fit"`: `escala = min(ancho / rango_x, alto / rango_y) * 0.9`
- Con `aspect: "stretch"`: `escala_x = ancho / rango_x`, `escala_y = alto / rango_y`

La transformación a píxeles es: `pixel_x = math_x * escala`, `pixel_y = -math_y * escala`.

---

## Example

```scalar
Resolution(800, 600)
Background(0.05, 0.05, 0.1)

// Ejes simples
Axes(-10, 10, -10, 10)

// Ejes con cuadrícula y colores personalizados
Axes(-5, 5, -5, 5,
    grid: true,
    tick_step: 1.0,
    axis_color: [0.7, 0.7, 0.7, 1.0],
    grid_color: [0.15, 0.15, 0.2, 1.0],
    axis_width: 2.5,
    arrows: true)

// Ejes animados con rebote
Axes(-6, 6, -4, 4,
    grid: true,
    animate: true,
    anim_duration: 2.0,
    anim_easing: "ease_out_bounce")
```
