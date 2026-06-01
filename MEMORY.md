# Scalar Project Memory

## Last Session
- **Date:** 2026-06-03
- **Summary:** Fixed WriteText double-spacing bug (cursor_x already baked into path coords). Added `DrawThenFill` AnimationKind variant (two-phase scale-up + fill-fade). Added standalone `DrawThenFill()` function for any node. Added `RevealText()` — character-by-character draw-then-fill text reveal. Updated demo script and wiki docs.
- **Changed files:**
  - `crates/scalar_bridge/src/lib.rs` — Added `AnimationKind::DrawThenFill { from_scale, fill_rgba }` variant; added initial capture + two-phase dispatch in `before_frame` (phase 1: scale 0→1 at 0–60% progress, phase 2: fill fade at 60–100%)
  - `crates/scalar_bridge/src/bindings/animation.rs` — Fixed WriteText double-spacing (removed cumulative advance offset from char_x); added `register_draw_then_fill()` (standalone DrawThenFill function); added `register_reveal_text()` (character-by-character draw-then-fill reveal); **BUG FIX**: `WriteText`/`RevealText` now read `x`/`y` from kwargs as fallback (not only positional args)
  - `crates/scalar_bridge/src/bindings/imports.rs` — **BUG FIX**: `Text()` now reads `x`/`y` from kwargs as fallback
  - `test_text_anim.scl` — Added RevealText demo section; updated duration hint from 12→14s
  - `wiki/lang/animation.md` — Added RevealText and DrawThenFill sections with full kwarg tables and examples; added "How Animations Work" lifecycle section
  - `wiki/bindings/animation_api.md` — Added DrawThenFill and RevealText API reference sections; updated implementation notes for DrawThenFill
  - `wiki/lang/grammar_spec.md` — Added DrawThenFill and RevealText to animation function table

## Open TODOs
- [ ] Download code and test on Windows PC with `test_text_anim.scl` (GPU required)
- [ ] Verify WriteText spacing is correct (no double spacing)
- [ ] Verify RevealText draw-then-fill visual effect
- [ ] Profile CPU hot spots: `sync_ecs_to_shape_batcher`, `before_frame`, `animator_system.run()`
- [ ] Run `test_anim.scl` and compare perf before/after
- [ ] Consider `--release` profile for benchmarking (debug mode is 10-50× slower for CPU work)
- [ ] Add coverage for composite glyphs (font parser handles composites, but test with accented chars)
- [ ] Consider adding kerning support for Text() (currently uses only advance width)
- [ ] Consider adding SVG `<rect>`, `<circle>`, `<ellipse>` element support in SVGImport
- [ ] Consider adding multiline text support (newline handling) to Text()
- [ ] Consider adding `TransformCharacters(source_ids, target_ids)` to morph one text into another

## Key Decisions

### Easing system as standalone module
Created `easing.rs` as its own module (not inside bindings/) so it's accessible by both `lib.rs` (before_frame) and `bindings/animation.rs` and `bindings/primitives.rs`. Follows unidirectional dependency flow.

### Animation via set_visible(false) instead of zero-length collapse
**Problem:** Collapsing lines to zero length (`update_line_endpoints(id, x1, y1, x1, y1)`) caused the shape batcher to render them as circles (ferrous_2d `line_with_cap` draws a circle when `length < 0.0001`).
**Fix:** Spawn lines at full length, hide with `set_visible(id, false)`, store `was_hidden: true` in `AnimatingLine`, reveal on first frame where `progress > 0`.

### Closure → standalone function for axes rendering
Replaced closure `spawn_axes_line` (which captured `&mut r`) with standalone function `spawn_line_animated(r: &mut Renderer, ...)` to avoid potential `RefMut` borrow edge cases with NLL.

### Plot animation staggering
Plot segments animate with stagger (each starts after the previous), using per-segment delay `segment * (anim_dur / samples)`. Segment duration is `anim_dur / samples * 1.5` for slight overlap and smoothness. Axes animation uses delay=0 (all lines in parallel).

### Zero-allocation render loop via buffer pool + threaded ffmpeg writer
**Problem:** Before this session, `poll_and_map` allocated a new 33 MB `Vec<u8>` per render. At 960 renders/sec (4K × 240fps × 4 sub-samples) that's 32 GB/s malloc/free churn. Additionally, scalar_cli allocated another `vec![0u32; buf_size]` per frame for accumulation.
**Fix:** Three-pronged approach:
1. `poll_and_map_into` — caller provides pre-allocated buffer, no allocation in readback path
2. Render buffer pool — sub-sample buffers allocated once, reused by clearing
3. Writer thread with buffer recycling — output buffers travel render→writer→pool→render, no deallocation needed
**Result:** Zero per-frame allocations in the hot render loop.

### Unified kwargs system for all shapes
All shape functions (Rect, Circle, Triangle, Star, RegularPolygon, Polygon, SVG) share a unified kwarg system:
- `fill` / `fill_color` — fill color (default white)
- `stroke` — stroke color (optional, no stroke if omitted)
- `stroke_width` — stroke thickness  
- `opacity` — global opacity multiplier (applied to both fill and stroke alpha)
- `z_index` — ECS z-ordering for draw order
- `rotation` — degrees (converted to radians, applied via `Quat::from_rotation_z()`)
- `visible` — visibility toggle
- `cap` — line cap for stroke rendering
This avoids per-shape kwarg duplication and makes the API predictable.

### SVG path parsing as built-in shape feature
Built a lightweight SVG path parser supporting all common commands (M/L/H/V/C/S/Q/T/Z in both absolute and relative forms). Quadratic beziers (Q/q, T/t) are converted to equivalent cubic beziers using the standard formula: CP1 = Q0 + 2/3*(Q1-Q0), CP2 = Q2 + 2/3*(Q1-Q2). This avoids needing a separate SVG library and keeps the dependency footprint minimal.

### All shapes go through spawn_2d_path
All filled shapes (Rect, Circle, Triangle, etc.) use `spawn_2d_path()` with explicit `PathCommand` sequences rather than the specialized `spawn_2d_rect()` / `spawn_2d_circle()` helper methods. This means rotation through Transform works uniformly for all shapes (the specialized helpers don't accept rotation). The path-based approach also enables future morphing between any two shapes since they all share the same `PathData` component type.

### Plot animation timing model with overlap control
Added `anim_delay` and `anim_overlap` to Plot() for full timeline control.
- `anim_delay` adds a leading pause before the first segment starts (useful for coordinating multiple plots).
- `anim_overlap` ∈ [0, 1] controls segment concurrency: 0 = fully sequential, 1 = fully parallel, 0.5 = smooth overlapping default.
- Formula: `segment_duration = total_dur / (1 + (n-1)*(1-overlap))`, `delay_between = segment_duration * (1-overlap)`.
- Segment generation refactored to first-collect-then-spawn pattern: all `LineData` collected in a Vec first, then lines are spawned and animation registered in a separate loop. This avoids borrowing issues and prepares for future enhancements.

### Text as vector paths, not rasterized
Text() renders glyph outlines as `PathCommand` sequences using the font's TrueType outlines, not via MSDF atlas or bitmap textures. This means text is resolution-independent, supports all shape kwargs (fill, stroke, rotation, opacity), and integrates with the existing 2D path rendering pipeline. No GPU font atlas is needed — the font file is only parsed for its outline data. Quadratic beziers in glyph outlines are converted to cubic using the same formula as the SVG parser.

### SVGImport uses simple XML scanner, not full XML parser
To avoid adding an XML parsing dependency, SVGImport uses a custom scanner that finds `<path>` tags and extracts attributes via pattern matching. This handles 90%+ of real-world SVGs (attributes in any order, double/single quotes, self-closing tags, namespace prefixes). Missing: `<rect>`, `<circle>`, `<g>` elements, CSS styling, transforms.

### NONE sentinel for "no fill"
Added a `NONE` constant (`Value::List(vec![])`) as a sentinel meaning "explicitly no fill/transparent". This is handled in `parse_shape_kwargs` — when fill is set to an empty list, `ShapeKwargs.fill` is set to `None` and `remove_fill()` is called on the renderer instead of `set_fill()`. SVG files with `fill="none"` also generate this sentinel.

### FontImport stores FontParser, not Font
The bridge stores `ferrous_font::parser::FontParser` objects (CPU-side font outline parser) rather than `ferrous_font::Font` (which includes a GPU atlas). Since text is rendered as vector paths, only the outline parser is needed. This avoids GPU resource management in the bridge and keeps font loading lightweight.

### Text baseline positioning
Text(x, y) uses (x, y) as the baseline start position (bottom-left of the first line). This is the standard typographic convention. Descenders (g, j, p, q, y) extend below y; the main body of text sits above y. Documented clearly in wiki.

### Wiki split into modular files
`grammar_spec.md` grew too large (246 lines) covering syntax + 5 function domains. Split into focused files under `wiki/lang/`:
- `syntax.md` — core language
- `axes.md` — Axes() reference
- `plot.md` — Plot() reference
- `shapes.md` — Line/Rect/Circle
- `project.md` — Resolution/Background/SetFPS/MotionBlur
- `animation.md` — Animate/SetLineProgress/SetLineCap
This follows the project's strict modularity tenet.

## Implementation Constraints
- **Do NOT run `cargo run -p scalar_cli` on this server.** The CLI requires a GPU / display server. The user downloads the code and runs on their Windows PC. Only `cargo check` / `cargo build` are safe here.

## Known Issues / Technical Debt
- ferrous_ui_core, ferrous_core, ferrous_engine, etc. have pre-existing warnings (unused imports, etc.) — not our concern
- svgparser v0.8.1 is flagged for future Rust incompatibility
- GPU at 16% utilization: CPU is the bottleneck
- `sync_ecs_to_shape_batcher` iterates ~900 entities/frame — main CPU hot spot candidate
- `before_frame` walks all active animations — second hot spot candidate
- `scalar_lang` has 2 pre-existing warnings (unused import in eval/mod.rs, dead_code in lexer.rs)

## Localization Debt
- [x] `crates/scalar_bridge/src/bindings/primitives.rs` — doc comments were in Spanish, now translated to English
- [x] `crates/scalar_bridge/src/bindings/shapes.rs` — top doc comment in Spanish, now translated to English
- [x] `crates/scalar_bridge/src/bindings/animation.rs` — inline comment on line 24 was in Spanish, translated during full rewrite on 2026-06-01

## Relevant Files
- `crates/scalar_bridge/src/easing.rs` — 30 easing functions
- `crates/scalar_bridge/src/lib.rs` — Bridge, AnimatingLine, before_frame
- `crates/scalar_bridge/src/bindings/primitives.rs` — **Axes()**, Plot()
- `crates/scalar_bridge/src/bindings/animation.rs` — Animate(), SetLineProgress(), SetLineCap()
- `crates/scalar_bridge/src/bindings/project.rs` — SetFPS(), MotionBlur()
- `crates/scalar_bridge/src/bindings/shapes.rs` — **Line(), Rect(), Circle(), Triangle(), Star(), RegularPolygon(), Polygon(), SVG()**
- `crates/scalar_bridge/src/bindings/imports.rs` — **SVGImport(), FontImport(), Text()**
- `test_anim.scl` — animated demo script
- `test_plot.scl` — static plot demo script
- `test_shapes.scl` — shapes demo script
- `test_text.scl` — text rendering demo script
- `crates/scalar_cli/src/main.rs` — threaded ffmpeg writer, buffer pool, motion blur
- `wiki/lang/grammar_spec.md` — language reference index
- `wiki/lang/axes.md` — full Axes() kwarg reference
- `wiki/lang/plot.md` — full Plot() kwarg reference
- `wiki/lang/syntax.md` — core syntax documentation
- `wiki/lang/shapes.md` — **all shape functions with unified kwargs**
- `wiki/lang/text.md` — **SVGImport, FontImport, Text reference**
- `wiki/lang/project.md` — project settings
- `wiki/lang/animation.md` — animation functions
