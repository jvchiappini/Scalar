# Scalar Project Memory

## Last Session
- Date: 2026-06-01 (continued)
- Summary: Implemented `Wait()` with timeline cursor — a thread-local `TIMELINE_CURSOR` that advances on each `Wait(t)` call. All animation functions automatically read the cursor in `parse_anim_params()` and add it to their `delay`. This enables declarative sequential choreography without manual delay arithmetic. Updated wiki docs, roadmap, and all tests pass.
- Changed files:
  - `crates/scalar_bridge/src/bindings/animation.rs` — ADDED `thread_local! { TIMELINE_CURSOR }`, modified `parse_anim_params()` to add cursor to delay, added `register_wait()` function
  - `wiki/lang/animation.md` — ADDED `Wait()` section with examples and timeline cursor note
  - `wiki/roadmap/_index.md` — UPDATED: `Wait()` marked ✅ done, `Play()` deferred ⏳, completed milestones updated with fn/if/return/comparison items
  - `MEMORY.md` — Updated session state

## Open TODOs
- [ ] **P1: `Rotate()` / `Spin()`** — rotation animation (`AnimationKind::Rotate`)
- [ ] **P1: `Arrow`, `Dot`, `NumberPlane`, `Brace`** — new shape objects
- [ ] **P0: `&&` / `||` logical operators** — compound conditions for `if`
- [ ] **P2+: See `wiki/roadmap/_index.md`** for full feature roadmap
- [ ] Test on Windows PC with `test_text_anim.scl` at `-d 11` to verify text morph works
- [ ] Add caching by expression string in `latex.rs`
- [ ] Fix `Grow` animation (currently a no-op — captures scale 1.0, animates to 1.0)
- [ ] Consider adding kerning support for Text()
- [ ] Add `return` at top level (currently unwrapped at fn-call boundary; top-level `return` returns sentinel)

## Key Decisions

### Wait() uses thread-local Cell<f64> instead of Rc<RefCell>
A `thread_local! { static TIMELINE_CURSOR: Cell<f64> }` approach avoids modifying every animation function's signature. `parse_anim_params()` reads the cursor and adds it to delay. This is invisible to callers — `Wait()` just sets the cursor, and all subsequent animation calls automatically inherit the offset.

### Play() deferred — not strictly necessary
Multiple animation calls without an intervening `Wait()` already run in parallel (same cursor value). The pattern `Wait(max_duration)` achieves the same effect as `Play()`. A native `Play()` wrapper that computes max duration from function results is a future improvement but not blocking.

### if/else as statement (not expression)
`Stmt::If` returns the last statement's value from the executed branch, making it behave like an expression when used as the last statement in a function. Simpler than mutual recursion between `expr` and `stmt` in the parser.

### Return as sentinel value
`Value::Return(Box<Value>)` propagates through statement evaluation (loops, if branches) and is unwrapped at the function-call boundary in `eval_expr`. Avoids adding a `Result<Value, ControlFlow>` error type.

### Functions as foundation
User-defined functions (`fn`) enable abstraction and reuse. Top-level-only for now (no nested functions). Lexical scoping via `Environment::fresh_child()`.

### Text() returns per-glyph [NodeId] like Tex()
Each glyph is a separate entity, enabling per-glyph animation (FadeIn each letter, morph each letter individually). Matches `Tex()` API.

## Implementation Constraints
- **Do NOT run `cargo run -p scalar_cli` on this server** unless explicitly testing a change. The CLI requires a GPU / display server. Prefer `cargo check` / `cargo test`.

## Known Issues / Technical Debt
- `test_lex` unit test is broken (pre-existing — logos lexer parses numbers as float bits)
- `align_point_sequences` function in `scalar_bridge/src/lib.rs` is unused (legacy from old morph approach)
- `Grow` animation is a no-op (captures current scale ~1.0, animates to 1.0)
- ferrous_ui_core, ferrous_core, ferrous_engine, etc. have pre-existing warnings (unused imports, etc.) — not our concern
- svgparser v0.8.1 is flagged for future Rust incompatibility
- GPU at 16% utilization: CPU is the bottleneck
- `sync_ecs_to_shape_batcher` iterates ~900 entities/frame — main CPU hot spot candidate
- `before_frame` walks all active animations — second hot spot candidate
- `scalar_lang` has 1 pre-existing warning (`dead_code` in lexer.rs)

## Localization Debt
- [x] `crates/scalar_bridge/src/bindings/primitives.rs` — doc comments translated to English
- [x] `crates/scalar_bridge/src/bindings/shapes.rs` — top doc comment translated to English
- [x] `crates/scalar_bridge/src/bindings/animation.rs` — inline comment translated during rewrite

## Relevant Files
- `crates/scalar_bridge/src/easing.rs` — 30 easing functions
- `crates/scalar_bridge/src/lib.rs` — Bridge, AnimationEntry, AnimationKind, before_frame, morph helpers
- `crates/scalar_bridge/src/bindings/animation.rs` — All animation functions + Wait()
- `crates/scalar_bridge/src/bindings/project.rs` — SetFPS(), MotionBlur()
- `crates/scalar_bridge/src/bindings/shapes.rs` — Line(), Rect(), Circle(), Triangle(), Star(), RegularPolygon(), Polygon(), SVG()
- `crates/scalar_bridge/src/bindings/imports.rs` — SVGImport(), FontImport(), Text()
- `ferrous_engine/src/renderer.rs` — set_path_data()
- `test_anim.scl` — animated demo script
- `test_text_anim.scl` — animated text/LaTeX demo script
- `crates/scalar_cli/src/main.rs` — threaded ffmpeg writer, buffer pool, motion blur
- `wiki/lang/animation.md` — animation functions + Wait() docs
- `wiki/lang/grammar_spec.md` — language reference index
- `wiki/lang/syntax.md` — core syntax documentation
- `wiki/lang/shapes.md` — all shape functions with unified kwargs
- `wiki/lang/text.md` — SVGImport, FontImport, Text reference
- `wiki/roadmap/_index.md` — full prioritized roadmap
