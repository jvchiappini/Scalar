# Scalar Project Memory

## Last Session
- Date: 2026-06-01 (continued)
- Summary: Refactored `Text()` to return per-glyph `[NodeId]` instead of single `NodeId`, enabling per-glyph text morphing (each letter morphs independently). Added text morph demo to `test_text_anim.scl` (Section 5: "Hello" → "World"). Created `wiki/roadmap/_index.md` with full prioritized roadmap. Updated wiki docs reflecting `Text()` return type change.
- Changed files:
  - `crates/scalar_bridge/src/bindings/imports.rs` — CHANGED: `Text()` now spawns one entity per glyph, returns `Value::List([NodeId, ...])`
  - `test_text_anim.scl` — ADDED Section 5: per-glyph text morph "Hello" → "World"
  - `wiki/roadmap/_index.md` — NEW: full prioritized roadmap (P0–P4)
  - `wiki/lang/text.md` — UPDATED: `Text()` return type `NodeId` → `[NodeId]`, "How It Works" updated, morph example added, Tex note updated
  - `wiki/lang/grammar_spec.md` — UPDATED: `Text()` return type changed
  - `wiki/lang/animation.md` — UPDATED: Morph docs with per-glyph text morph example
  - `MEMORY.md` — Updated session state and open TODOs

## Open TODOs
- [ ] **P0: Implement user-defined functions (`fn`)** — AST, parser, runtime, evaluator
- [ ] **P0: Implement `Wait()` + `Play()`** — timeline-based choreography sequencing
- [ ] **P1: Implement `Rotate()` / `Spin()`** — rotation animation
- [ ] **P1: `Arrow`, `Dot`, `NumberPlane`, `Brace`** — new shape objects
- [ ] **P2+: See `wiki/roadmap/_index.md`** for full feature roadmap
- [ ] Test on Windows PC with `test_text_anim.scl` at `-d 11` to verify text morph works
- [ ] Add caching by expression string in `latex.rs`
- [ ] Fix `Grow` animation (currently a no-op — captures scale 1.0, animates to 1.0)
- [ ] Consider adding kerning support for Text()
- [ ] Profile CPU hot spots

## Key Decisions

### Text() returns per-glyph [NodeId] like Tex()
Each glyph in a `Text()` call is now spawned as a separate entity, enabling per-glyph animation (FadeIn each letter, morph each letter individually). This matches the API of `Tex()`, which already returns `[NodeId]` per display item. Backward compatible: all animation functions already accept `[NodeId]` via `parse_node_ids()`.

### Roadmap prioritized by impact/effort
Created `wiki/roadmap/_index.md` with P0–P4 priorities. Functions (`fn`) first because they enable abstraction and make every subsequent feature easier to test and demo. `Wait()` + `Play()` second because they replace manual delay arithmetic. Both are foundation for professional scripting.

### Functions as foundation for everything
User-defined functions (`fn`) change the language qualitatively: scripts can be organized, reused, and composed. They also enable the pattern library approach (pre-built demo components) and are required for complex multi-scene demos. Implementation touches AST, parser, runtime, and evaluator — but is self-contained and well-understood.

## Known Issues / Technical Debt
- (same as before, plus:)
- `test_lex` unit test is broken (pre-existing — logos lexer parses numbers as float bits)
- `align_point_sequences` function in `scalar_bridge/src/lib.rs` is unused (legacy from old morph approach)
- `Grow` animation is a no-op (captures current scale ~1.0, animates to 1.0)

## Implementation Constraints
- **Do NOT run `cargo run -p scalar_cli` on this server.** The CLI requires a GPU / display server. The user downloads the code and runs on their Windows PC. Only `cargo check` / `cargo build` are safe here.

## Known Issues / Technical Debt
- ferrous_ui_core, ferrous_core, ferrous_engine, etc. have pre-existing warnings (unused imports, etc.) — not our concern
- svgparser v0.8.1 is flagged for future Rust incompatibility
- GPU at 16% utilization: CPU is the bottleneck
- `sync_ecs_to_shape_batcher` iterates ~900 entities/frame — main CPU hot spot candidate
- `before_frame` walks all active animations — second hot spot candidate
- `scalar_lang` has 2 pre-existing warnings (unused import in eval/mod.rs, dead_code in lexer.rs)
- `Grow` animation is a no-op (captures current scale ~1.0, animates to 1.0)

## Localization Debt
- [x] `crates/scalar_bridge/src/bindings/primitives.rs` — doc comments were in Spanish, now translated to English
- [x] `crates/scalar_bridge/src/bindings/shapes.rs` — top doc comment in Spanish, now translated to English
- [x] `crates/scalar_bridge/src/bindings/animation.rs` — inline comment on line 24 was in Spanish, translated during full rewrite on 2026-06-01

## Relevant Files
- `crates/scalar_bridge/src/easing.rs` — 30 easing functions
- `crates/scalar_bridge/src/lib.rs` — Bridge, AnimationEntry, AnimationKind, before_frame, morph helpers
- `crates/scalar_bridge/src/bindings/animation.rs` — Animate(), Morph(), SetLineProgress(), SetLineCap()
- `crates/scalar_bridge/src/bindings/project.rs` — SetFPS(), MotionBlur()
- `crates/scalar_bridge/src/bindings/shapes.rs` — **Line(), Rect(), Circle(), Triangle(), Star(), RegularPolygon(), Polygon(), SVG()**
- `crates/scalar_bridge/src/bindings/imports.rs` — **SVGImport(), FontImport(), Text()**
- `ferrous_engine/src/renderer.rs` — set_path_data()
- `test_anim.scl` — animated demo script
- `test_text_anim.scl` — animated text/LaTeX demo script
- `crates/scalar_cli/src/main.rs` — threaded ffmpeg writer, buffer pool, motion blur
- `wiki/lang/animation.md` — animation functions (Animate, Morph, FadeIn/Out, etc.)
- `wiki/lang/grammar_spec.md` — language reference index
- `wiki/lang/syntax.md` — core syntax documentation
- `wiki/lang/shapes.md` — **all shape functions with unified kwargs**
- `wiki/lang/text.md` — **SVGImport, FontImport, Text reference**
