pub mod math_eval;
pub mod bindings;
pub mod easing;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::Environment;
use ferrous_engine::{Renderer, RendererMode};

/// Metadata stored for each line, used by progress-based animations.
#[derive(Clone, Debug)]
pub struct LineData {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// A registered line animation entry.
pub struct AnimatingLine {
    pub node_id: u64,
    pub duration: f64,
    pub delay: f64,
    pub start_time: Option<f64>,
    pub easing: easing::Easing,
    /// If true, the element was hidden before animation starts.
    /// The system will show it on the first frame where progress > 0.
    pub was_hidden: bool,
}

pub struct Bridge {
    pub renderer: Rc<RefCell<Renderer>>,
    /// Stores original endpoints for lines created via `Line()`.
    pub line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    /// Active line-draw animations.
    pub animations: Rc<RefCell<Vec<AnimatingLine>>>,
    /// Output frames per second (overridable via `SetFPS()` in script).
    pub fps: Rc<RefCell<u32>>,
    /// Motion blur sub-samples (0 = disabled). Set via `MotionBlur(samples)` in script.
    pub motion_blur_samples: Rc<RefCell<u32>>,
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

        // Default to Pure 2D — no 3D world sync, orthographic camera.
        renderer.gpu.mode = RendererMode::Pure2D;
        renderer.gpu.enable_pure_2d(ferrous_engine::wgpu::Color::BLACK);

        Ok(Self {
            renderer: Rc::new(RefCell::new(renderer)),
            line_data: Rc::new(RefCell::new(HashMap::new())),
            animations: Rc::new(RefCell::new(Vec::new())),
            fps: Rc::new(RefCell::new(fps)),
            motion_blur_samples: Rc::new(RefCell::new(0)),
        })
    }

    pub fn register_functions(&self, env: &mut Environment) {
        bindings::project::register(env, self.renderer.clone(), self.fps.clone(), self.motion_blur_samples.clone());
        bindings::shapes::register(env, self.renderer.clone(), self.line_data.clone());
        bindings::animation::register(env, self.renderer.clone(), self.line_data.clone(), self.animations.clone());
        bindings::primitives::register(env, self.renderer.clone(), self.line_data.clone(), self.animations.clone());
    }

    /// Called before each frame render. Advances all active line-draw animations.
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

            // Initialize start_time for animations first seen this frame
            if anim.start_time.is_none() {
                anim.start_time = Some(time + anim.delay);
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
                let _ = renderer.set_visible(
                    ferrous_engine::NodeId(anim.node_id.into()),
                    true,
                );
            }

            // Apply easing curve to the raw progress
            let eased_progress = easing::apply(&anim.easing, progress as f64) as f32;

            if let Some(data) = line_data.get(&anim.node_id) {
                let end_x = data.x1 + (data.x2 - data.x1) * eased_progress;
                let end_y = data.y1 + (data.y2 - data.y1) * eased_progress;
                let _ = renderer.update_line_endpoints(
                    ferrous_engine::NodeId(anim.node_id.into()),
                    data.x1, data.y1, end_x, end_y,
                );
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
