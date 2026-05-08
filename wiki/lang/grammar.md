# Scalar Grammar Specification (v14)

## Named Arguments (Kwargs)
Scalar now supports named arguments in function and method calls. Keyword arguments must follow any positional arguments.

**Syntax:**
`func(pos1, pos2, key1: value1, key2: value2)`

**Example:**
```scalar
let c = Circle(1.5)
c.morph_to(other, duration: 2.0, easing: "ease_in_out")
```

## Functions and Variables
- `let x = value`: Variable declaration.
- `func(...)`: Global function call.
- `obj.method(...)`: Method call on an object.

## Flow Control
- `for i in start..end { ... }`: Range-based for loop.
