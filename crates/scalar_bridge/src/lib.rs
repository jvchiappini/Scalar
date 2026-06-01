pub mod math_eval;
pub mod ratex_render;
pub mod bindings;
pub mod easing;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::Environment;
use ferrous_engine::{Renderer, RendererMode, NodeId};
use ferrous_engine::glam::Vec3;
use ferrous_engine::Transform;
use crate::bindings::imports::FontEntry;

/// Metadata stored for each line, used by progress-based animations.
#[derive(Clone, Debug)]
pub struct LineData {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Type of animation to apply each frame.
#[derive(Debug, Clone)]
pub enum AnimationKind {
    /// Line-draw reveal: endpoint interpolates from start to end.
    /// Element is hidden before animation starts (handled by `was_hidden` on the entry).
    LineDraw,
    /// Opacity fade: lerp from `from_opacity` to `to_opacity`.
    Fade { from_opacity: f32, to_opacity: f32 },
    /// Uniform scale: lerp from `from` to `to`.
    /// `from` is lazily captured from the current transform on the first frame.
    Scale { from: Option<f32>, to: f32 },
    /// Position move: lerp from `(from_x,from_y)` to `(to_x,to_y)`.
    /// `from_*` is lazily captured on the first frame.
    MoveTo { from_x: Option<f32>, from_y: Option<f32>, to_x: f32, to_y: f32 },
    /// Two-phase "draw then fill" reveal: phase 1 (0→60% eased progress) scales from
    /// 0→1 with fill transparent; phase 2 (60→100%) holds scale at 1 and fades fill in
    /// from transparent→original color. `from_scale` is lazily captured on first frame.
    DrawThenFill { from_scale: Option<f32>, fill_rgba: [f32; 4] },
    /// Path-by-path progressive draw then fill reveal (for text/vector outlines).
    /// Phase 1 (0–60% eased progress): shows sub-path segments one by one along the
    /// outline. Phase 2 (60–100%): all segments visible, fill fades in.
    PathDrawThenFill {
        /// Entity IDs for each stroke segment sub-path (hidden initially).
        segment_ids: Vec<u64>,
        /// Target fill color (0–1 range).
        fill_rgba: [f32; 4],
        /// The main entity that holds the fill (hidden initially, no stroke).
        fill_entity_id: u64,
        /// First-frame initializer flag (sets fill transparent).
        initialized: bool,
    },
    /// Path morphing: interpolates the node's path commands from `source_points`
    /// to `target_points`. Both are aligned (padded to the same length) at
    /// registration time.
    ///
    /// When the morph starts (first progress > 0), the `source_ids` nodes are
    /// automatically hidden so they don't overlap with the morphing result.
    ///
    /// When the morph completes, `restore_cmds` are written back to the target
    /// node so the final shape is pixel-perfect (not a polyline approximation).
    Morph {
        source_points: Vec<ferrous_engine::glam::Vec2>,
        target_points: Vec<ferrous_engine::glam::Vec2>,
        /// Node IDs to hide when the morph starts (the source shape).
        source_ids: Vec<u64>,
        /// Original target path commands to restore on completion.
        restore_cmds: Vec<ferrous_engine::PathCommand>,
    },
}

/// A registered animation entry.
pub struct AnimationEntry {
    pub node_id: u64,
    pub duration: f64,
    pub delay: f64,
    pub start_time: Option<f64>,
    pub easing: easing::Easing,
    pub kind: AnimationKind,
    /// If true, the element starts hidden and is shown on first progress > 0.
    /// Used by LineDraw and any animation that needs a hidden-to-visible transition.
    pub was_hidden: bool,
}

pub struct Bridge {
    pub renderer: Rc<RefCell<Renderer>>,
    /// Stores original endpoints for lines created via `Line()`.
    pub line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    /// Active animations (line draws, fades, scales, moves).
    pub animations: Rc<RefCell<Vec<AnimationEntry>>>,
    /// Output frames per second (overridable via `SetFPS()` in script).
    pub fps: Rc<RefCell<u32>>,
    /// Motion blur sub-samples (0 = disabled). Set via `MotionBlur(samples)` in script.
    pub motion_blur_samples: Rc<RefCell<u32>>,
    /// Loaded fonts for text rendering (indexed by FontImport return value).
    pub fonts: Rc<RefCell<Vec<FontEntry>>>,
}

impl Bridge {
    /// Creates a new Bridge owning a headless Renderer initialized in Pure2D mode.
    pub fn new(width: u32, height: u32, fps: u32) -> anyhow::Result<Self> {
        let mut renderer = Renderer::builder()
            .with_dimensions(width, height)
            .with_headless_mode(true)
            .with_fps(fps)
            .build()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Use Full3D so `sync_world` runs and lyon_tessellation processes Path elements.
        // Pure2D skips the WorldPass and uses a simple batcher that draws beziers as straight lines.
        renderer.gpu.mode = RendererMode::Full3D;
        renderer.gpu.camera_system.camera.set_mode_2d(true, Some(height as f32));
        renderer.gpu.camera_system.camera.set_aspect(width as f32 / height as f32);
        renderer.gpu.set_background_color(ferrous_engine::wgpu::Color::BLACK);
        renderer.gpu.camera_system.set_tonemapping_enabled(&renderer.gpu.context.queue, false);

        Ok(Self {
            renderer: Rc::new(RefCell::new(renderer)),
            line_data: Rc::new(RefCell::new(HashMap::new())),
            animations: Rc::new(RefCell::new(Vec::new())),
            fps: Rc::new(RefCell::new(fps)),
            motion_blur_samples: Rc::new(RefCell::new(0)),
            fonts: Rc::new(RefCell::new(Vec::new())),
        })
    }

    pub fn register_functions(&self, env: &mut Environment) {
        bindings::project::register(env, self.renderer.clone(), self.fps.clone(), self.motion_blur_samples.clone());
        bindings::shapes::register(env, self.renderer.clone(), self.line_data.clone());
        bindings::animation::register(env, self.renderer.clone(), self.line_data.clone(), self.animations.clone(), self.fonts.clone());
        bindings::primitives::register(env, self.renderer.clone(), self.line_data.clone(), self.animations.clone());
        bindings::imports::register(env, self.renderer.clone(), self.fonts.clone());
        bindings::latex::register(env, self.renderer.clone(), self.fonts.clone());
    }

    /// Called before each frame render. Advances all active animations
    /// (line draws, fades, scales, moves).
    pub fn before_frame(&self, time: f64) {
        let mut anims = self.animations.borrow_mut();
        if anims.is_empty() {
            return;
        }

        let line_data = self.line_data.borrow();
        let mut renderer = self.renderer.borrow_mut();

        // Process all animations and remove completed ones
        let mut i = 0;
        while i < anims.len() {
            let anim = &mut anims[i];

            // Initialize start_time for animations first seen this frame.
            // For Scale / MoveTo, also capture the initial transform.
            if anim.start_time.is_none() {
                anim.start_time = Some(time + anim.delay);

                match &mut anim.kind {
                    AnimationKind::Scale { ref mut from, .. } => {
                        if from.is_none() {
                            if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                                if let Some(t) = renderer.world.ecs.get::<Transform>(*entity) {
                                    *from = Some((t.scale.x + t.scale.y + t.scale.z) / 3.0);
                                }
                            }
                        }
                    }
                    AnimationKind::MoveTo {
                        ref mut from_x,
                        ref mut from_y,
                        ..
                    } => {
                        if from_x.is_none() {
                            if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                                if let Some(t) = renderer.world.ecs.get::<Transform>(*entity) {
                                    *from_x = Some(t.position.x);
                                    *from_y = Some(t.position.y);
                                }
                            }
                        }
                    }
                    AnimationKind::DrawThenFill { ref mut from_scale, fill_rgba } => {
                        if from_scale.is_none() {
                            if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                                if let Some(t) = renderer.world.ecs.get::<Transform>(*entity) {
                                    *from_scale = Some((t.scale.x + t.scale.y + t.scale.z) / 3.0);
                                }
                            }
                            // Make fill fully transparent at start
                            let _ = renderer.set_fill(
                                NodeId(anim.node_id.into()),
                                [fill_rgba[0], fill_rgba[1], fill_rgba[2], 0.0],
                            );
                        }
                    }
                    AnimationKind::Fade { from_opacity, .. } => {
                        // Set initial opacity immediately (during the delay period,
                        // the element should already be at from_opacity).
                        let _ = renderer.set_opacity(
                            NodeId(anim.node_id.into()),
                            *from_opacity,
                        );
                    }
                    AnimationKind::PathDrawThenFill { ref mut initialized, fill_rgba, fill_entity_id, .. } => {
                        if !*initialized {
                            *initialized = true;
                            // Make fill transparent on the fill entity at start
                            let _ = renderer.set_fill(
                                NodeId(*fill_entity_id),
                                [fill_rgba[0], fill_rgba[1], fill_rgba[2], 0.0],
                            );
                        }
                    }
                    AnimationKind::Morph { source_ids, .. } => {
                        // Hide the source shape nodes when morph starts
                        // so they don't overlap with the morphing target.
                        for sid in source_ids {
                            let _ = renderer.set_visible(NodeId(*sid), false);
                        }
                    }
                    _ => {}
                }
            }

            let start = anim.start_time.expect("already initialized above");
            let elapsed = time - start;
            let progress = if elapsed <= 0.0 {
                0.0
            } else {
                (elapsed / anim.duration).clamp(0.0, 1.0) as f32
            };

            // Show hidden element on the first frame where progress > 0
            if progress > 0.0 && anim.was_hidden {
                anim.was_hidden = false;
                let _ = renderer.set_visible(NodeId(anim.node_id.into()), true);
            }

            // Only apply animation effects when progress > 0.
            // At progress == 0, the element should stay in its default state
            // (visible, full opacity, etc.). This is critical for animations
            // whose delay exceeds the render duration — without this guard,
            // FadeIn would set opacity to 0.0 on every frame and the element
            // would remain permanently invisible.
            if progress > 0.0 {
                // Apply easing curve to the raw progress
                let eased = easing::apply(&anim.easing, progress as f64) as f32;

                // Dispatch by animation kind
                match &anim.kind {
                AnimationKind::LineDraw => {
                    if let Some(data) = line_data.get(&anim.node_id) {
                        let end_x = data.x1 + (data.x2 - data.x1) * eased;
                        let end_y = data.y1 + (data.y2 - data.y1) * eased;
                        let _ = renderer.update_line_endpoints(
                            NodeId(anim.node_id.into()),
                            data.x1, data.y1, end_x, end_y,
                        );
                    }
                }
                AnimationKind::Fade { from_opacity, to_opacity } => {
                    let opacity = from_opacity + (to_opacity - from_opacity) * eased;
                    let _ = renderer.set_opacity(NodeId(anim.node_id.into()), opacity);
                }
                AnimationKind::Scale { from, to } => {
                    if let Some(from_s) = from {
                        let scale = from_s + (to - from_s) * eased;
                        if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                            if let Some(t) = renderer.world.ecs.get::<Transform>(*entity) {
                                let new_t = Transform {
                                    position: t.position,
                                    rotation: t.rotation,
                                    scale: Vec3::new(scale, scale, scale),
                                };
                                let _ = renderer.update_transform(
                                    NodeId(anim.node_id.into()),
                                    new_t,
                                );
                            }
                        }
                    }
                }
                AnimationKind::MoveTo {
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                } => {
                    if let (Some(fx), Some(fy)) = (from_x, from_y) {
                        let x = fx + (to_x - fx) * eased;
                        let y = fy + (to_y - fy) * eased;
                        if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                            if let Some(t) = renderer.world.ecs.get::<Transform>(*entity) {
                                let new_t = Transform {
                                    position: Vec3::new(x, y, t.position.z),
                                    rotation: t.rotation,
                                    scale: t.scale,
                                };
                                let _ = renderer.update_transform(
                                    NodeId(anim.node_id.into()),
                                    new_t,
                                );
                            }
                        }
                    }
                }
                AnimationKind::DrawThenFill { from_scale, fill_rgba } => {
                    const PHASE_SPLIT: f32 = 0.6;
                    if eased <= PHASE_SPLIT {
                        // Phase 1: scale from 0→1, fill stays transparent
                        if let Some(from) = from_scale {
                            let t = eased / PHASE_SPLIT;
                            let s = from + (1.0 - from) * t;
                            if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                                if let Some(tr) = renderer.world.ecs.get::<Transform>(*entity) {
                                    let new_t = Transform {
                                        position: tr.position,
                                        rotation: tr.rotation,
                                        scale: Vec3::new(s, s, s),
                                    };
                                    let _ = renderer.update_transform(
                                        NodeId(anim.node_id.into()),
                                        new_t,
                                    );
                                }
                            }
                        }
                    } else {
                        // Phase 2: scale stays at 1, fill fades in
                        let t = (eased - PHASE_SPLIT) / (1.0 - PHASE_SPLIT);
                        let alpha = t * fill_rgba[3];
                        let _ = renderer.set_fill(
                            NodeId(anim.node_id.into()),
                            [fill_rgba[0], fill_rgba[1], fill_rgba[2], alpha],
                        );
                        // Ensure scale is pinned at 1.0
                        if let Some(entity) = renderer.node_map.get(&NodeId(anim.node_id.into())) {
                            if let Some(tr) = renderer.world.ecs.get::<Transform>(*entity) {
                                let avg = (tr.scale.x + tr.scale.y + tr.scale.z) / 3.0;
                                if (avg - 1.0).abs() > 0.01 {
                                    let new_t = Transform {
                                        position: tr.position,
                                        rotation: tr.rotation,
                                        scale: Vec3::ONE,
                                    };
                                    let _ = renderer.update_transform(
                                        NodeId(anim.node_id.into()),
                                        new_t,
                                    );
                                }
                            }
                        }
                    }
                }
                AnimationKind::PathDrawThenFill { segment_ids, fill_rgba, fill_entity_id, initialized: _ } => {
                    const SPLIT: f32 = 0.6;
                    if eased <= SPLIT {
                        // Phase 1: reveal sub-path segments one by one along the outline.
                        let seg_progress = (eased / SPLIT).min(1.0);
                        let total = segment_ids.len();
                        let num_to_show = if total == 0 { 0 } else {
                            (seg_progress * total as f32).floor() as usize
                        };
                        for (i, sid) in segment_ids.iter().enumerate() {
                            let _ = renderer.set_visible(NodeId(*sid), i < num_to_show);
                        }
                    } else {
                        let _ = renderer.set_visible(NodeId(*fill_entity_id), true);
                        let fill_t = ((eased - SPLIT) / (1.0 - SPLIT)).min(1.0);
                        let alpha = fill_t * fill_rgba[3];
                        let _ = renderer.set_fill(
                            NodeId(*fill_entity_id),
                            [fill_rgba[0], fill_rgba[1], fill_rgba[2], alpha],
                        );
                        for sid in segment_ids {
                            let _ = renderer.set_visible(NodeId(*sid), true);
                        }
                    }
                }
                AnimationKind::Morph { source_points, target_points, .. } => {
                    // Linearly interpolate each point: result = lerp(source, target, eased)
                    use ferrous_engine::glam::Vec2;
                    let interp: Vec<Vec2> = source_points
                        .iter()
                        .zip(target_points.iter())
                        .map(|(s, t)| Vec2::lerp(*s, *t, eased))
                        .collect();
                    let pd = points_to_path_data(&interp);
                    let _ = renderer.set_path_data(NodeId(anim.node_id.into()), pd);
                }
                } // match &anim.kind
            }

            if progress < 1.0 {
                i += 1; // keep this animation
            } else {
                // Before removing, restore the original target path for Morph
                // so the final shape is pixel-perfect (not a polyline approx).
                if let AnimationKind::Morph { restore_cmds, .. } = &anim.kind {
                    let pd = ferrous_engine::PathData { commands: restore_cmds.clone() };
                    let _ = renderer.set_path_data(NodeId(anim.node_id.into()), pd);
                }
                anims.swap_remove(i); // remove completed, don't increment i
            }
        }
    }

    /// Internal: updates line_data when a new line is spawned.
    pub fn track_line(&self, node_id: u32, data: LineData) {
        self.line_data.borrow_mut().insert(node_id as u64, data);
    }
}

// ── Morph helpers ──────────────────────────────────────────────────────────

/// Evaluates a cubic Bézier curve at parameter `t`.
fn cubic_bezier_point(
    p0: ferrous_engine::glam::Vec2,
    p1: ferrous_engine::glam::Vec2,
    p2: ferrous_engine::glam::Vec2,
    p3: ferrous_engine::glam::Vec2,
    t: f32,
) -> ferrous_engine::glam::Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3
}

/// Samples a path uniformly at `num_samples` evenly-spaced points.
///
/// Cubics are subdivided into 8-line-segment approximations so the sampling
/// is smooth. `MoveTo` and `Close` are handled correctly — `Close` closes
/// back to the most recent `MoveTo`.
///
/// This produces a clean, evenly-spaced polyline regardless of the original
/// path's vertex density. Circle ↔ Rect morphs look smooth because both
/// get the same number of uniformly-spaced sample points.
fn sample_path_uniformly(
    cmds: &[ferrous_engine::PathCommand],
    num_samples: usize,
) -> Vec<ferrous_engine::glam::Vec2> {
    use ferrous_engine::PathCommand::*;
    use ferrous_engine::glam::Vec2;

    if num_samples == 0 || cmds.is_empty() {
        return vec![Vec2::ZERO; num_samples.max(1)];
    }

    // Step 1: build dense polyline (flatten curves)
    let mut polyline: Vec<Vec2> = Vec::new();
    let mut cursor = Vec2::ZERO;
    let mut start = Vec2::ZERO;
    const CURVE_STEPS: usize = 8;

    for cmd in cmds {
        match cmd {
            MoveTo(p) => {
                cursor = *p;
                start = *p;
                polyline.push(*p);
            }
            LineTo(p) => {
                polyline.push(*p);
                cursor = *p;
            }
            CubicTo(c1, c2, p) => {
                for i in 1..=CURVE_STEPS {
                    let t = i as f32 / CURVE_STEPS as f32;
                    polyline.push(cubic_bezier_point(cursor, *c1, *c2, *p, t));
                }
                cursor = *p;
            }
            Close => {
                if cursor.distance_squared(start) > 0.0001 {
                    polyline.push(start);
                }
                cursor = start;
            }
        }
    }

    // Step 2: cumulative segment lengths
    let n_pts = polyline.len();
    let mut cum_len = vec![0.0f32; n_pts];
    for i in 1..n_pts {
        cum_len[i] = cum_len[i - 1] + polyline[i].distance(polyline[i - 1]);
    }
    let total_len = cum_len[n_pts - 1];

    if total_len < 0.0001 {
        return vec![polyline[0]; num_samples];
    }

    // Step 3: sample evenly-spaced points
    let mut samples = Vec::with_capacity(num_samples);
    let mut seg_idx = 0;
    for s in 0..num_samples {
        let target_dist = (s as f32 / (num_samples - 1).max(1) as f32) * total_len;
        // Advance segment index until we reach or pass target_dist
        while seg_idx + 1 < n_pts - 1 && cum_len[seg_idx + 1] < target_dist {
            seg_idx += 1;
        }
        if seg_idx + 1 >= n_pts {
            samples.push(polyline[n_pts - 1]);
        } else {
            let seg_start = cum_len[seg_idx];
            let seg_len = cum_len[seg_idx + 1] - seg_start;
            let t = if seg_len > 0.0 {
                ((target_dist - seg_start) / seg_len).clamp(0.0, 1.0)
            } else {
                0.0
            };
            samples.push(polyline[seg_idx].lerp(polyline[seg_idx + 1], t));
        }
    }
    samples
}

/// Aligns two point sequences to the same length by padding the shorter one
/// with its last point.
fn align_point_sequences(
    a: &[ferrous_engine::glam::Vec2],
    b: &[ferrous_engine::glam::Vec2],
) -> (Vec<ferrous_engine::glam::Vec2>, Vec<ferrous_engine::glam::Vec2>) {
    use ferrous_engine::glam::Vec2;
    let max_len = a.len().max(b.len());
    let pad = |v: &[Vec2], len: usize| -> Vec<Vec2> {
        if v.len() >= len {
            v.to_vec()
        } else {
            let last = v.last().copied().unwrap_or(Vec2::ZERO);
            let mut out = v.to_vec();
            out.resize(len, last);
            out
        }
    };
    (pad(a, max_len), pad(b, max_len))
}

/// Reconstructs a `PathData` from linearly-interpolated points.
/// Result: MoveTo(first_pt), LineTo(intermediate_pts...), Close.
fn points_to_path_data(pts: &[ferrous_engine::glam::Vec2]) -> ferrous_engine::PathData {
    use ferrous_engine::PathCommand::*;
    let cmds = if pts.is_empty() {
        vec![]
    } else {
        let mut cmds = vec![MoveTo(pts[0])];
        for p in &pts[1..] {
            cmds.push(LineTo(*p));
        }
        cmds.push(Close);
        cmds
    };
    ferrous_engine::PathData { commands: cmds }
}

/// Extracts a flat `Vec<u64>` of node IDs from a Scalar `Value`.
///
/// Accepts:
/// - `Value::NodeId(id)` → `vec![id]`
/// - `Value::Number(n)` → `vec![n as u64]`
/// - `Value::List([NodeId, ...])` → all NodeIds and Numbers in the list
pub fn value_to_ids(val: &scalar_lang::Value) -> Result<Vec<u64>, String> {
    use scalar_lang::Value;
    match val {
        Value::List(list) => {
            let ids: Vec<u64> = list.iter().filter_map(|v| match v {
                Value::NodeId(id) => Some(*id as u64),
                Value::Number(n) => Some(*n as u64),
                _ => None,
            }).collect();
            if ids.is_empty() {
                Err("list contains no valid node IDs".to_string())
            } else {
                Ok(ids)
            }
        }
        Value::NodeId(id) => Ok(vec![*id as u64]),
        Value::Number(n) => Ok(vec![*n as u64]),
        _ => Err("expected a NodeId, Number, or [NodeId]".to_string()),
    }
}

/// Looks up a single node by ID and returns its `PathCommand` list.
pub fn node_to_path_commands(
    id: u64,
    renderer: &ferrous_engine::Renderer,
) -> Result<Vec<ferrous_engine::PathCommand>, String> {
    let entity = renderer.node_map.get(&NodeId(id))
        .ok_or_else(|| format!("node {} not found", id))?;
    let pd = renderer.world.ecs.get::<ferrous_engine::PathData>(*entity)
        .ok_or_else(|| format!("node {} has no PathData", id))?;
    Ok(pd.commands.clone())
}
