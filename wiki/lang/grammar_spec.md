# Scalar Grammar Specification

This document defines the formal grammar and syntax for the Scalar Language.

## Syntax Overview

Scalar is a declarative-execution language designed for visual mathematics and animation.

### Core Types
- **Number**: 64-bit floating point.
- **List**: Ordered collection of values.
- **NodeId**: Reference to an entity in the engine.
- **Color**: RGBA vector.

### Statements
- **Variable Declaration**: `let x = 10;`
- **For Loop**: `for i in 0..10 { ... }`
- **Method Call**: `object.method(args);`
- **Import**: `import "filename.scl";` (Merges functions and variables with the current scope)

### Methods & Animation
- `animate(property, target, duration, easing)`: Schedules an animation job.
- `set_position(x, y, z)`: Sets the position of a node.
- `set_color(r, g, b, a)`: Sets the color.

### Vector Typography
- `Text("Hello", "font_name", size)`: Creates a vector text node.

## Object-Oriented Syntax
Scalar supports method injection. If `obj` is a `NodeId`, calling `obj.animate(...)` is equivalent to a native call where `obj` is the first argument.

```scalar
let my_circle = Circle(50);
my_circle.set_position(0, 0, 0);
my_circle.animate("radius", 100, 2.0, "EaseInOut");
```
