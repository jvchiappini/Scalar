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
| For Loop | `for i in 0..10 { ... }` | Iterates over a numeric range |
| Method Call | `object.method(args)` | Calls a method on a NodeId |
| Import | `import "filename.scl"` | Executes another script, merging its scope |
| Expression | `Axes(-5, 5, -3, 3)` | Any expression evaluated as a statement |

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
