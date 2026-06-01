# Scalar & Ferrous Engine — Roadmap

> **Mission:** Build a pure-Rust deterministic animation framework that matches
> Manim's expressiveness and surpasses it in performance, portability, and
> zero-dependency deployment.

---

## Priority Matrix

| Priority | Feature | Impact | Effort | Area |
|----------|---------|--------|--------|------|
| 🔥 P0 | **Functions (`fn`)** | Foundation for abstraction, reuse, libraries | Medium | Language |
| 🔥 P0 | **`Wait()` + `Play()`** | Professional choreography sequencing | Medium | Animation |
| 🔥 P1 | **`Arrow` / `Dot` / `NumberPlane`** | Most-missed Manim objects | Medium | Shapes |
| 🔥 P1 | **`Rotate()` / `Spin()`** | Essential animation missing today | Small | Animation |
| 🧠 P2 | **`ValueTracker` + binding** | Parametric animation, Manim's magic | Large | Lang+ECS |
| 🧠 P2 | **`WaitUntil()` / timeline triggers** | Event-driven sequencing | Medium | Animation |
| 🧠 P2 | **`VGroup` / `arrange()` / `next_to()`** | Auto-layout, scene composition | Large | Shapes |
| 🧠 P2 | **`Brace()`** | Math annotation | Small | Shapes |
| 🔮 P3 | **Camera: zoom, pan, move_to** | Cinematic effects | Large | Engine |
| 🔮 P3 | **`Ellipse` / `Arc` / `Angle`** | Geometry toolkit | Medium | Shapes |
| 🔮 P3 | **`Lambda` / `map()` / `filter()`** | Functional programming | Medium | Language |
| 🔮 P3 | **`LaggedStart` / `LaggedStartMap`** | Advanced stagger patterns | Small | Animation |
| 🔮 P4 | **`Transform` (curved path)** | Non-linear morph trajectories | Medium | Animation |
| 🔮 P4 | **Loop animation (`forever`)** | Cyclic animations (pulsing, spinning) | Medium | Animation |
| 🔮 P4 | **`Scene` objects / layers** | Manim-style scene management | Large | Architecture |
| 🔮 P4 | **3D scene support (camera orbit)** | Beyond Manim's 2.5D | Very Large | Engine |

---

## Phase 1 — Language & Choreography (Current)

### P0: Functions (`fn`) — Milestone 1

**Goal:** Add user-defined functions to Scalar so scripts can be organized, reused, and composed.

```scalar
fn title(text, y) {
    let t = Text(text, 0, y, font: font, size: 28, fill: [0.5, 0.5, 0.8, 1.0])
    FadeIn(t, duration: 0.3)
    return t
}

fn morph_demo(source, target, label, delay) {
    title(label, 280)
    SetVisibility(target, false)
    Animate(target, "Morph", into: source, duration: 1.0, delay: delay)
}
```

**Implementation steps:**
1. Add `Stmt::FnDef` to AST
2. Add `Value::Fn` to runtime
3. Parse `fn name(params) { body }`
4. Evaluate `FnDef` — store function in environment
5. Evaluate `Call` — support `Value::Fn` (user functions) alongside `Value::NativeFunction`
6. Tests: define, call, return, scope isolation, recursion

**Files affected:** `ast.rs`, `runtime.rs`, `parser.rs`, `eval_expr.rs`, `eval_stmt.rs`

---

### P0: `Wait()` + `Play()` — Milestone 2

**Goal:** Replace manual `delay` arithmetic with declarative sequencing.

```scalar
Play(
    FadeIn(circle, duration: 1.0),
    Grow(rect, duration: 1.5)
)
Wait(0.5)
Sequence(
    FadeIn(star, duration: 1.0),
    FadeIn(triangle, duration: 0.5)
)
```

**Design:** A timeline-based execution model where `Play()` blocks the time cursor until its animations complete. `Wait()` advances the cursor. `Sequence()` chains animations back-to-back.

**Implementation steps:**
1. Add a simulated-time cursor to the evaluator
2. `Play()` registers animations and advances the time cursor to `max(end_times)`
3. `Wait(t)` advances time cursor by `t`
4. `Sequence(anim1, anim2)` runs anim1, then when done, runs anim2

**Key insight:** The time cursor is separate from wall clock. The renderer still uses its own deterministic frame timing; the evaluator just pre-computes delays.

---

## Phase 2 — Object Library (Next)

### P1: New Shapes

| Shape | Signature | Notes |
|-------|-----------|-------|
| `Arrow(x1, y1, x2, y2, ...)` | Points, vector annotation | Tip size, double-headed options |
| `Dot(x, y, radius, ...)` | Small filled circle marker | Fixed-radius marker |
| `NumberPlane(x_range, y_range, ...)` | Math grid with numbers | Axes + grid + tick labels |
| `Brace(obj_or_points, direction, ...)` | Curly brace annotation | Auto-size, label support |

### P1: `Rotate()` / `Spin()`

```scalar
Rotate(circle, 360, duration: 2.0)  // smooth rotation
Spin(star, revolutions: 3, duration: 2.0)  // continuous spin
```

**Implementation:** `AnimationKind::Rotate { from_angle, to_angle }` in ECS. Interpolates the transform rotation. `Spin` = `Rotate` with `to_angle = from_angle + 360 * revolutions`.

---

## Phase 3 — Advanced (Future)

| Feature | Description | Why |
|---------|-------------|-----|
| `ValueTracker` + `Bind` | Animate an independent variable, auto-update dependent objects | Manim's most powerful abstraction |
| `VGroup` / `arrange()` | Group objects, auto-layout (vertical/horizontal/grid) | Scene composition |
| `next_to()` / `shift()` | Relative positioning without manual coordinates | Layout ergonomics |
| Camera system | Zoom, pan, follow camera | Cinematic effects |
| Easing library expansion | Bounce, elastic, back, cubic bezier | More natural motion |
| `Lambda`, `map`, `filter` | First-class functions + collections | Data-driven scenes |
| `Transform` with curved paths | Morph along bezier trajectories | More complex morphs |
| Looping animation (`forever`) | Pulsing, orbiting, continuous effects | Cyclic demos |
| Scene graph / layers | Named scenes, configurable layers | Large productions |

---

## Engineering Constants

These are non-negotiable and apply to every feature:

- ✅ **Strict Modularity** — No God Files. Split when a module grows beyond one responsibility.
- ✅ **Impeccable Rustdoc** — Every `pub` item documented with `///`.
- ✅ **Absolute Determinism** — Frame N renders identically every run. No wall-clock dependency.
- ✅ **Unidirectional Dependency** — `ferrous_engine` must NEVER import `scalar_lang` or `scalar_bridge`.
- ✅ **English Only** — All docs, comments, wiki, error messages in English.
- ✅ **Wiki Sync** — Every API change must update the corresponding `/wiki` file.
- ✅ **No `.unwrap()`** in library code. Only in test helpers and CLI entry points.

---

## Completed Milestones

- [x] **RaTeX integration** — Pure-Rust LaTeX math renderer (~99.5% KaTeX coverage)
- [x] **Per-glyph Tex()** — `Tex()` returns `[NodeId]`, one per display item
- [x] **Per-glyph Text()** — `Text()` returns `[NodeId]`, one per glyph
- [x] **Animate() universal dispatcher** — String-based effect dispatch with stagger
- [x] **Morph() system** — Uniform path sampling, per-glyph morph, source auto-hide, target restore
- [x] **SetVisibility()** — Show/hide nodes by ID (single or list)
- [x] **For-each loops** — `for x in list { }`
- [x] **Variable assignment** — `x = expr`
- [x] **Binary operators** — `+`, `-`, `*`, `/` with precedence
- [x] **String escapes** — Lexer handles `\\`, `\"`, `\n`, `\t`, `\r`
