# Scalar Syntax Reference

Core language syntax, types, and conventions.

## Core Types

| Type | Description | Example |
|------|-------------|---------|
| **Number** | 64-bit floating point | `42`, `-3.14`, `1.5e10` |
| **String** | Text in double quotes | `"hello"` |
| **Boolean** | Truth value | `true`, `false` |
| **List** | Ordered collection | `[1, 2, 3]`, `[0.5, 0.3, 0.8, 1.0]` |
| **NodeId** | Render entity reference (returned by shape constructors) | `let l = Line(...)` → `l` is a NodeId |
| **Color** | RGBA vector (4-element List or individual components) | `[1.0, 0.3, 0.3, 1.0]`, or `RED` |

## Statements

| Statement | Syntax | Description |
|-----------|--------|-------------|
| Variable Declaration | `let x = 10` | Binds a value to a name |
| Assignment | `x = x + 1` | Reassigns an existing variable (must be previously declared) |
| For Loop (range) | `for i in 0..10 { ... }` | Iterates `i` from `0` to `9` (exclusive end) |
| For Loop (list) | `for item in list_expr { ... }` | Iterates over each element in a List value |
| Method Call | `object.method(args)` | Calls a method on a NodeId |
| Import | `import "filename.scl"` | Executes another script, merging its scope |
| Expression | `Axes(-5, 5, -3, 3)` | Any expression evaluated as a statement |

## Binary Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `delay + 0.5` |
| `-` | Subtraction | `end - start` |
| `*` | Multiplication | `i * 0.08` |
| `/` | Division | `total / count` |

Precedence (highest to lowest): `*` `/` (multiplicative), then `+` `-` (additive).

```scalar
let delay = 19.5
let step = 0.08
for part in parts {
    FadeIn(part, duration: 0.25, delay: delay)
    delay = delay + step          // Assignment + binary op
}
```

## Named Arguments (Kwargs)

Functions accept keyword arguments after positional arguments:

```scalar
func(pos1, pos2, key1: value1, key2: value2)
```

## Object-Oriented Syntax

If `obj` is a `NodeId`, calling `obj.method(...)` is equivalent to a native function call where `obj` is the first argument.

```scalar
let c = Circle(50)
c.set_fill(RED)        // Sets fill color
c.set_stroke(1, 1, 1, 1, 2.0)  // Sets stroke
c.set_z_index(10)      // Sets render order
```

## Standard Colors

| Name | RGBA |
|------|------|
| `WHITE` | `[1.0, 1.0, 1.0, 1.0]` |
| `BLACK` | `[0.0, 0.0, 0.0, 1.0]` |
| `RED` | `[1.0, 0.0, 0.0, 1.0]` |
| `GREEN` | `[0.0, 1.0, 0.0, 1.0]` |
| `BLUE` | `[0.0, 0.0, 1.0, 1.0]` |
| `YELLOW` | `[1.0, 1.0, 0.0, 1.0]` |
| `CYAN` | `[0.0, 1.0, 1.0, 1.0]` |
| `MAGENTA` | `[1.0, 0.0, 1.0, 1.0]` |
