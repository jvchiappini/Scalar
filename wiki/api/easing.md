# Easing Functions

Easing functions (curvas de aceleración) controlan la velocidad de una animación a lo largo del tiempo. Se usan en `Axes()`, `Plot()`, y `Animate()`.

## Uso

```scalar
// Como kwarg en Axes o Plot:
Axes(-6, 6, -3, 3, animate: true, anim_easing: "ease_out_cubic")
Plot("sin(x)", -6, 6, animate: true, anim_easing: "ease_in_out_quart")

// Como kwarg en Animate:
Animate(lines: [l1, l2, l3], per_line: 1.0, easing: "ease_out_bounce")
```

## Catálogo de Easing

### Básicos
| Nombre | Descripción | Curva |
|--------|-------------|-------|
| `linear` | Velocidad constante | t |
| `ease_in_quad` | Acelera suavemente | t² |
| `ease_out_quad` | Desacelera suavemente | t(2−t) |
| `ease_in_out_quad` | Acelera y desacelera | 2t² para t<0.5 |

### Cúbicos / Cuárticos / Quínticos
| Nombre | Descripción |
|--------|-------------|
| `ease_in_cubic` / `ease_out_cubic` / `ease_in_out_cubic` | Potencia 3 |
| `ease_in_quart` / `ease_out_quart` / `ease_in_out_quart` | Potencia 4 |
| `ease_in_quint` / `ease_out_quint` / `ease_in_out_quint` | Potencia 5 |

### Trigonométricos
| Nombre | Descripción |
|--------|-------------|
| `ease_in_sine` / `ease_out_sine` / `ease_in_out_sine` | Curva sinusoidal |

### Exponenciales
| Nombre | Descripción |
|--------|-------------|
| `ease_in_expo` / `ease_out_expo` / `ease_in_out_expo` | 2^(10(t−1)) |

### Circulares
| Nombre | Descripción |
|--------|-------------|
| `ease_in_circ` / `ease_out_circ` / `ease_in_out_circ` | Arco de círculo |

### Elásticos (overshoot + oscilación)
| Nombre | Descripción |
|--------|-------------|
| `ease_in_elastic` / `ease_out_elastic` / `ease_in_out_elastic` | Rebote elástico |

### Back (overshoot sin oscilación)
| Nombre | Descripción |
|--------|-------------|
| `ease_in_back` / `ease_out_back` / `ease_in_out_back` | Retrocede antes de avanzar |

### Bounce (rebote)
| Nombre | Descripción |
|--------|-------------|
| `ease_in_bounce` / `ease_out_bounce` / `ease_in_out_bounce` | Rebota al final |

## Formato

Los nombres son case-insensitive y aceptan guiones:
- `"ease_out_cubic"`, `"easeOutCubic"`, `"ease-out-cubic"` → todos válidos
- Por defecto: `"ease_out_cubic"`

## Implementación

Ver `crates/scalar_bridge/src/easing.rs` para las funciones matemáticas.
