pub mod math_eval;
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
            }

            if progress < 1.0 {
                i += 1; // keep this animation
            } else {
                anims.swap_remove(i); // remove completed, don't increment i
            }
        }
    }

    /// Internal: updates line_data when a new line is spawned.
    pub fn track_line(&self, node_id: u32, data: LineData) {
        self.line_data.borrow_mut().insert(node_id as u64, data);
    }
}
