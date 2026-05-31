use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::Renderer;

/// Registers project-level functions into the Scalar environment.
///
/// Exposed functions:
/// - `Resolution(width, height)` — resize the render target
/// - `Background(r, g, b)` or `Background(r, g, b, a)` — set background clear color
/// - `SetFPS(fps)` — override output frame rate
/// - `MotionBlur(samples)` — enable/disable motion blur (0 = off, >0 = sub-samples)
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
}
