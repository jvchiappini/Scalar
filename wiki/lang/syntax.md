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
| **Function Definition** | `fn name(params) { body }` | Defines a reusable user function |
| **Conditional** | `if cond { body } else { body }` | Conditional execution (else is optional) |
| **Return** | `return expr?` | Early return from a function |

## Functions (continued)

- `return` statement — early return from any point in a function
- `if/else` — conditional execution with truthiness semantics

```scalar
fn factorial(n) {
    if n <= 1 {
        return 1          // early return
    } else {
        n * factorial(n - 1)   // recursive call
    }
}
factorial(5)  // returns 120
```

### `if` / `else`

```scalar
if condition { ... } else { ... }
```

- `else` is optional (if omitted and condition is false, the if statement evaluates to `0`)
- **Truthiness**: `Number(0)` and `Boolean(false)` are falsy. Everything else is truthy.
- The `if` statement evaluates to the last expression of the executed branch

```scalar
// Simple conditional
if x > 0 {
    Text("Positive", 0, 0)
}

// With else
if score >= 90 {
    grade = "A"
} else {
    grade = "B"
}
```

### `return` Statement

```scalar
return expr
return          // returns 0.0
```

- Immediately exits the current function and returns the given value
- If no expression is given, returns `0.0`
- Works inside loops and nested `if` blocks

```scalar
fn find_first(items, target) {
    for x in items {
        if x == target {
            return x    // exits early
        }
    }
    return 0            // not found
}
```

### Limitations (current)

- No nested function definitions (yet)

## Binary Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `delay + 0.5` |
| `-` | Subtraction | `end - start` |
| `*` | Multiplication | `i * 0.08` |
| `/` | Division | `total / count` |
| `<` | Less than | `x < 10` |
| `<=` | Less than or equal | `n <= 1` |
| `>` | Greater than | `score > 90` |
| `>=` | Greater than or equal | `x >= 0` |
| `==` | Equal | `x == 42` |
| `!=` | Not equal | `x != 0` |

Comparison operators return `Boolean` (`true` or `false`).

Precedence (highest to lowest): `*` `/` (multiplicative), then `+` `-` (additive), then `<` `<=` `>` `>=` `==` `!=` (comparison).

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
