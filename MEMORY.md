# Scalar Project Memory

## Last Session
- Date: 2026-05-31
- Summary: Optimized CPU-side allocation churn in render loop. Added zero-allocation GPU readback (`poll_and_map_into`), rewrote scalar_cli with buffer pool + threaded ffmpeg writer. GPU at 16% — bottleneck confirmed CPU-side.
- Changed files:
  - `../96a0b3c0-be13-4354-8674-72220ab22596/crates/ferrous_renderer/src/resources/readback.rs` — Added `poll_and_map_into(&self, device, &mut Vec<u8>)` which reuses caller's buffer instead of allocating a new `Vec<u8>` per frame. Old `poll_and_map` delegates to it.
  - `../96a0b3c0-be13-4354-8674-72220ab22596/crates/ferrous_engine/src/renderer.rs` — `render_frame_into` now uses `poll_and_map_into` to write into the caller-provided buffer.
  - `crates/scalar_cli/src/main.rs` — **Complete rewrite:**
    - Pre-allocated `render_bufs` pool (one per motion-blur sub-sample, `Vec::with_capacity(buf_size)`, cleared between frames)
    - Pre-allocated `accum` buffer (`Vec<u32>`, cleared between frames)
    - Writer thread decoupled via `mpsc::sync_channel(4)` for frame output + `mpsc::channel()` return channel to recycle buffers back to pool
    - No per-frame `vec!` or `Vec::with_capacity` calls in hot loop
    - Motion blur path sends averaged output to writer thread without cloning
    - Progress indicator with renders/sec and MPixels/sec
    - Backpressure via sync_channel: render loop paces itself to ffmpeg writer speed
    - Fixed: changed `0..sub_samples-1` to `0..sub_samples` in motion blur loop (off-by-one)

## Open TODOs
- [ ] Profile CPU hot spots: `sync_ecs_to_shape_batcher`, `before_frame`, `animator_system.run()`
- [ ] Run `test_anim.scl` and compare perf before/after
- [ ] Consider `--release` profile for benchmarking (debug mode is 10-50× slower for CPU work)

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

## Known Issues / Technical Debt
- ferrous_ui_core, ferrous_core, ferrous_engine, etc. have pre-existing warnings (unused imports, etc.) — not our concern
- svgparser v0.8.1 is flagged for future Rust incompatibility
- GPU at 16% utilization: CPU is the bottleneck
- `sync_ecs_to_shape_batcher` iterates ~900 entities/frame — main CPU hot spot candidate
- `before_frame` walks all active animations — second hot spot candidate

## Relevant Files
- `crates/scalar_bridge/src/easing.rs` — 30 easing functions
- `crates/scalar_bridge/src/lib.rs` — Bridge, AnimatingLine, before_frame
- `crates/scalar_bridge/src/bindings/primitives.rs` — Axes(), Plot()
- `crates/scalar_bridge/src/bindings/animation.rs` — Animate(), SetLineProgress(), SetLineCap()
- `crates/scalar_bridge/src/bindings/project.rs` — SetFPS(), MotionBlur()
- `test_anim.scl` — animated demo script
- `crates/scalar_cli/src/main.rs` — threaded ffmpeg writer, buffer pool, motion blur
- `../96a0b3c0-be13-4354-8674-72220ab22596/crates/ferrous_renderer/src/resources/readback.rs` — `poll_and_map_into`
- `../96a0b3c0-be13-4354-8674-72220ab22596/crates/ferrous_engine/src/renderer.rs` — `render_frame_into`
