use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::Renderer;
use ferrous_engine::NodeId;

/// Registers project-level functions into the Scalar environment.
///
/// Exposed functions:
/// - `Resolution(width, height)` — resize the render target
/// - `Background(r, g, b)` or `Background(r, g, b, a)` — set background clear color
/// - `SetFPS(fps)` — override output frame rate
/// - `MotionBlur(samples)` — enable/disable motion blur (0 = off, >0 = sub-samples)
/// - `SetVisibility(node_id, visible)` — show/hide a node
pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    fps: Rc<RefCell<u32>>,
    motion_blur_samples: Rc<RefCell<u32>>,
) {
    // ─── Resolution(width, height) ─────────────────────────────────────────
    {
        let ren = renderer.clone();
        env.define(
            "Resolution".to_string(),
            Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
                if args.len() < 2 {
                    return Err("Resolution(width, height) requires 2 arguments".to_string());
                }
                let w = match &args[0] {
                    Value::Number(n) => *n as u32,
                    _ => return Err("Resolution: width must be a number".to_string()),
                };
                let h = match &args[1] {
                    Value::Number(n) => *n as u32,
                    _ => return Err("Resolution: height must be a number".to_string()),
                };
                let mut r = ren.borrow_mut();
                r.on_resize(w, h);
                Ok(Value::Number(0.0))
            })),
        );
    }

    // ─── Background(r, g, b [, a]) ────────────────────────────────────────
    {
        let ren = renderer.clone();
        env.define(
            "Background".to_string(),
            Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
                if args.len() < 3 {
                    return Err("Background(r, g, b [, a]) requires at least 3 arguments".to_string());
                }
                let r = match &args[0] {
                    Value::Number(n) => *n as f64,
                    _ => return Err("Background: r must be a number".to_string()),
                };
                let g = match &args[1] {
                    Value::Number(n) => *n as f64,
                    _ => return Err("Background: g must be a number".to_string()),
                };
                let b = match &args[2] {
                    Value::Number(n) => *n as f64,
                    _ => return Err("Background: b must be a number".to_string()),
                };
                let a = if args.len() >= 4 {
                    match &args[3] {
                        Value::Number(n) => *n as f64,
                        _ => 1.0,
                    }
                } else {
                    1.0
                };

                let mut renderer = ren.borrow_mut();
                renderer.gpu.set_background_color(ferrous_engine::wgpu::Color { r, g, b, a });
                Ok(Value::Number(0.0))
            })),
        );
    }

    // ─── SetFPS(fps) ────────────────────────────────────────────────────────
    {
        let fps_cell = fps.clone();
        env.define(
            "SetFPS".to_string(),
            Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
                let f = match args.get(0) {
                    Some(Value::Number(n)) if *n >= 1.0 => *n as u32,
                    _ => return Err("SetFPS(fps): fps must be a number >= 1".to_string()),
                };
                *fps_cell.borrow_mut() = f;
                Ok(Value::Number(0.0))
            })),
        );
    }

    // ─── MotionBlur(samples) ────────────────────────────────────────────────
    {
        let mb_cell = motion_blur_samples.clone();
        env.define(
            "MotionBlur".to_string(),
            Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
                let s = match args.get(0) {
                    Some(Value::Number(n)) if *n >= 0.0 => *n as u32,
                    _ => return Err("MotionBlur(samples): samples must be a number >= 0 (0 = off)".to_string()),
                };
                *mb_cell.borrow_mut() = s;
                Ok(Value::Number(0.0))
            })),
        );
    }

    // ─── SetVisibility(node_id, visible) ─────────────────────────────────────
    // Accepts a single NodeId, a Number, or a List of NodeIds (e.g. from Tex())
    {
        let ren = renderer.clone();
        env.define(
            "SetVisibility".to_string(),
            Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
                let visible = match args.get(1) {
                    Some(Value::Boolean(b)) => *b,
                    _ => return Err("SetVisibility(node_id, visible): second argument must be a Boolean".to_string()),
                };

                // Collect all node IDs from the first argument
                let ids: Vec<u64> = match args.get(0) {
                    Some(Value::List(list)) => {
                        list.iter().filter_map(|v| match v {
                            Value::NodeId(id) => Some(*id as u64),
                            Value::Number(n) => Some(*n as u64),
                            _ => None,
                        }).collect()
                    }
                    Some(Value::NodeId(id)) => vec![*id as u64],
                    Some(Value::Number(n)) => vec![*n as u64],
                    _ => return Err("SetVisibility(node_id, visible): first argument must be a NodeId, Number, or [NodeId]".to_string()),
                };

                if ids.is_empty() {
                    return Err("SetVisibility: no valid node IDs found".to_string());
                }

                let mut r = ren.borrow_mut();
                for id in &ids {
                    let _ = r.set_visible(NodeId(*id), visible);
                }
                Ok(Value::Number(ids.len() as f64))
            })),
        );
    }
}
