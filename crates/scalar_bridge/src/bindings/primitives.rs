//! # Bindings de primitivas matemáticas para Scalar
//!
//! ## Axes
//! `Axes(x_min, x_max, y_min, y_max [, grid:, tick_step:, axis_color:, grid_color:, axis_width:, arrows:, aspect:, animate:, anim_duration:, anim_easing:])`
//!
//! Dibuja ejes cartesianos con ticks, grid opcional y flechas.
//! Las coordenadas son matemáticas (no pixels) — el escalado al viewport es automático.
//!
//! ## Plot
//! `Plot("expression", x_min, x_max [, samples:, thickness:, color:, cap:, animate:, anim_duration:, anim_easing:])`
//!
//! Evalúa y dibuja una función matemática `f(x)` muestreando en `[x_min, x_max]`.

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::Renderer;

use crate::math_eval;
use crate::{LineData, AnimatingLine, easing};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => *n,
        _ => 0.0,
    }
}

/// Resuelve un color desde kwargs ("color:", "axis_color:", etc.) o devuelve default.
fn kwarg_color(kwargs: &HashMap<String, Value>, key: &str, default: [f32; 4]) -> [f32; 4] {
    match kwargs.get(key) {
        Some(Value::List(list)) => {
            let r = num(list.get(0)) as f32;
            let g = num(list.get(1)) as f32;
            let b = num(list.get(2)) as f32;
            let a = if list.len() >= 4 { num(list.get(3)) as f32 } else { 1.0 };
            [r, g, b, a]
        }
        _ => default,
    }
}

/// Resuelve un número desde kwargs.
fn kwarg_num(kwargs: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
    match kwargs.get(key) {
        Some(Value::Number(n)) => *n,
        _ => default,
    }
}

/// Resuelve un booleano desde kwargs.
fn kwarg_bool(kwargs: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    match kwargs.get(key) {
        Some(Value::Boolean(b)) => *b,
        _ => default,
    }
}

/// Resuelve un string desde kwargs.
fn kwarg_str<'a>(kwargs: &'a HashMap<String, Value>, key: &str, default: &'a str) -> &'a str {
    match kwargs.get(key) {
        Some(Value::String(s)) => s.as_str(),
        _ => default,
    }
}

/// Spawn a 2D line, optionally registering it for animation (hidden until revealed).
fn spawn_line_animated(
    r: &mut Renderer,
    x1: f32, y1: f32, x2: f32, y2: f32,
    width: f32, color: [f32; 4],
    animate: bool,
    line_data: &Rc<RefCell<HashMap<u64, LineData>>>,
    animations: &Rc<RefCell<Vec<AnimatingLine>>>,
    anim_dur: f64,
    anim_easing: easing::Easing,
) {
    let id = r.spawn_2d_line(x1, y1, x2, y2, width);
    let _ = r.set_stroke(id, color, width);
    if animate {
        let nid = id.0 as u64;
        line_data.borrow_mut().insert(nid, LineData { x1, y1, x2, y2 });
        animations.borrow_mut().push(AnimatingLine {
            node_id: nid,
            duration: anim_dur,
            delay: 0.0,
            start_time: None,
            easing: anim_easing,
            was_hidden: true,
        });
        // Hide line immediately — the animation system will reveal it
        let _ = r.set_visible(id, false);
    }
}

// ─── Public registration ───────────────────────────────────────────────────────

pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimatingLine>>>,
) {
    register_axes(env, renderer.clone(), line_data.clone(), animations.clone());
    register_plot(env, renderer.clone(), line_data.clone(), animations.clone());
}

// ─── Axes ─────────────────────────────────────────────────────────────────────

fn register_axes(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimatingLine>>>,
) {
    let ren = renderer.clone();
    let ld = line_data.clone();
    let an = animations.clone();
    env.define(
        "Axes".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 4 {
                return Err("Axes(x_min, x_max, y_min, y_max) requires 4 arguments".to_string());
            }
            let x_min = num(args.get(0)) as f32;
            let x_max = num(args.get(1)) as f32;
            let y_min = num(args.get(2)) as f32;
            let y_max = num(args.get(3)) as f32;

            let grid = kwarg_bool(&kwargs, "grid", false);
            let tick_step = kwarg_num(&kwargs, "tick_step", 0.0);
            let arrows = kwarg_bool(&kwargs, "arrows", true);
            let aspect = kwarg_str(&kwargs, "aspect", "fit").to_string();

            let axis_color = kwarg_color(&kwargs, "axis_color", [0.5, 0.5, 0.5, 1.0]);
            let grid_color = kwarg_color(&kwargs, "grid_color", [0.2, 0.2, 0.25, 1.0]);
            let axis_width = kwarg_num(&kwargs, "axis_width", 2.0) as f32;
            let tick_len = kwarg_num(&kwargs, "tick_len", 6.0) as f32;

            // Animation kwargs
            let animate_axes = kwarg_bool(&kwargs, "animate", false);
            let anim_dur = kwarg_num(&kwargs, "anim_duration", 1.5);
            let anim_easing = easing::Easing::from_str(kwarg_str(&kwargs, "anim_easing", "ease_out_cubic"));

            let mut r = ren.borrow_mut();
            let (w, h) = (r.width() as f32, r.height() as f32);

            // Calcular escala: mapea rango matemático a pixels
            let math_w = x_max - x_min;
            let math_h = y_max - y_min;
            if math_w <= 0.0 || math_h <= 0.0 {
                return Err("Axes: x_max must be > x_min and y_max > y_min".to_string());
            }

            let (scale_x, scale_y) = if aspect == "stretch" {
                (w / math_w, h / math_h)
            } else {
                // "fit" — maintain aspect ratio, add 10% margin
                let s = (w / math_w).min(h / math_h) * 0.9;
                (s, s)
            };

            // Centrar origen en la pantalla
            let origin_x = 0.0;  // centro de pantalla (Pure2D)
            let origin_y = 0.0;

            // Transformar math → pixel
            let to_pixel = |mx: f32, my: f32| -> (f32, f32) {
                (
                    origin_x + mx * scale_x,
                    origin_y - my * scale_y,  // Y invertida en pantalla
                )
            };

            // Use a macro-like pattern: inline calls to spawn_line_animated
            // (not a closure, to avoid any borrow/lifetime issues with &mut r)

            // ── Grid (detrás de todo) ──
            if grid {
                let step = if tick_step > 0.0 { tick_step as f32 } else {
                    auto_tick_step(math_w.max(math_h))
                };
                // Grid vertical
                let mut gx = (x_min / step).ceil() * step;
                while gx <= x_max {
                    if gx.abs() > step * 0.01 { // skip near origin
                        let (px, _) = to_pixel(gx, 0.0);
                        let (_, py0) = to_pixel(gx, y_min);
                        let (_, py1) = to_pixel(gx, y_max);
                        spawn_line_animated(&mut r, px, py0, px, py1, 1.0, grid_color,
                            animate_axes, &ld, &an, anim_dur, anim_easing);
                    }
                    gx += step;
                }
                // Grid horizontal
                let mut gy = (y_min / step).ceil() * step;
                while gy <= y_max {
                    if gy.abs() > step * 0.01 {
                        let (_, py) = to_pixel(0.0, gy);
                        let (px0, _) = to_pixel(x_min, gy);
                        let (px1, _) = to_pixel(x_max, gy);
                        spawn_line_animated(&mut r, px0, py, px1, py, 1.0, grid_color,
                            animate_axes, &ld, &an, anim_dur, anim_easing);
                    }
                    gy += step;
                }
            }

            // ── Eje X ──
            if y_min <= 0.0 && y_max >= 0.0 {
                let (px0, py) = to_pixel(x_min, 0.0);
                let (px1, _) = to_pixel(x_max, 0.0);
                spawn_line_animated(&mut r, px0, py, px1, py, axis_width, axis_color,
                    animate_axes, &ld, &an, anim_dur, anim_easing);
            }

            // ── Eje Y ──
            if x_min <= 0.0 && x_max >= 0.0 {
                let (px, py0) = to_pixel(0.0, y_min);
                let (_, py1) = to_pixel(0.0, y_max);
                spawn_line_animated(&mut r, px, py0, px, py1, axis_width, axis_color,
                    animate_axes, &ld, &an, anim_dur, anim_easing);
            }

            // ── Ticks ──
            let step = if tick_step > 0.0 { tick_step as f32 } else {
                auto_tick_step(math_w.max(math_h))
            };

            // Ticks en X
            let (_, y0_px) = to_pixel(0.0, 0.0);
            let mut tx = (x_min / step).ceil() * step;
            while tx <= x_max {
                if tx.abs() > step * 0.01 {
                    let (px, _) = to_pixel(tx, 0.0);
                    spawn_line_animated(&mut r, px, y0_px - tick_len * 0.5, px, y0_px + tick_len * 0.5, 1.5, axis_color,
                        animate_axes, &ld, &an, anim_dur, anim_easing);
                }
                tx += step;
            }
            // Ticks en Y
            let (x0_px, _) = to_pixel(0.0, 0.0);
            let mut ty = (y_min / step).ceil() * step;
            while ty <= y_max {
                if ty.abs() > step * 0.01 {
                    let (_, py) = to_pixel(0.0, ty);
                    spawn_line_animated(&mut r, x0_px - tick_len * 0.5, py, x0_px + tick_len * 0.5, py, 1.5, axis_color,
                        animate_axes, &ld, &an, anim_dur, anim_easing);
                }
                ty += step;
            }

            // ── Flechas ──
            if arrows {
                let (tip_x, tip_y) = to_pixel(x_max, 0.0);
                let (base_x, base_y) = to_pixel(x_max - math_w * 0.02, 0.0);
                draw_arrow_animated(&mut r, tip_x, tip_y, base_x, base_y, axis_color, axis_width,
                    animate_axes, &ld, &an, anim_dur, anim_easing);
                let (tip_x, tip_y) = to_pixel(0.0, y_max);
                let (base_x, base_y) = to_pixel(0.0, y_max - math_h * 0.02);
                draw_arrow_animated(&mut r, tip_x, tip_y, base_x, base_y, axis_color, axis_width,
                    animate_axes, &ld, &an, anim_dur, anim_easing);
            }

            Ok(Value::Number(0.0))
        })),
    );
}

/// Dibuja una flecha simple (2 líneas en ángulo), con soporte opcional de animación.
fn draw_arrow_animated(
    r: &mut Renderer,
    tx: f32, ty: f32,
    bx: f32, by: f32,
    color: [f32; 4],
    width: f32,
    animate: bool,
    line_data: &Rc<RefCell<HashMap<u64, LineData>>>,
    animations: &Rc<RefCell<Vec<AnimatingLine>>>,
    anim_dur: f64,
    anim_easing: easing::Easing,
) {
    let dx = tx - bx;
    let dy = ty - by;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 { return; }
    let nx = dx / len;
    let ny = dy / len;
    let spread = 8.0 + width * 2.0;

    // Left barb
    let lx = tx - nx * spread + ny * spread * 0.4;
    let ly = ty - ny * spread - nx * spread * 0.4;
    spawn_line_animated(r, tx, ty, lx, ly, width.max(1.5), color,
        animate, line_data, animations, anim_dur, anim_easing);

    // Right barb
    let rx = tx - nx * spread - ny * spread * 0.4;
    let ry = ty - ny * spread + nx * spread * 0.4;
    spawn_line_animated(r, tx, ty, rx, ry, width.max(1.5), color,
        animate, line_data, animations, anim_dur, anim_easing);
}

/// Elige un paso de tick razonable para un rango dado.
fn auto_tick_step(range: f32) -> f32 {
    let raw = range / 10.0;
    let magnitude = 10.0_f32.powf(raw.abs().log10().floor());
    let normalized = raw / magnitude;
    let nice = if normalized <= 1.5 { 1.0 }
               else if normalized <= 3.5 { 2.0 }
               else if normalized <= 7.5 { 5.0 }
               else { 10.0 };
    nice * magnitude
}

// ─── Plot ─────────────────────────────────────────────────────────────────────

fn register_plot(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimatingLine>>>,
) {
    let ren = renderer.clone();
    let ld = line_data.clone();
    let an = animations.clone();
    env.define(
        "Plot".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 3 {
                return Err("Plot(\"expression\", x_min, x_max) requires 3 arguments".to_string());
            }

            let expr_str = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("Plot: first argument must be a string expression".to_string()),
            };
            let x_min = num(args.get(1)) as f64;
            let x_max = num(args.get(2)) as f64;

            if x_max <= x_min {
                return Err("Plot: x_max must be > x_min".to_string());
            }

            let samples = kwarg_num(&kwargs, "samples", 200.0) as usize;
            let thickness = kwarg_num(&kwargs, "thickness", 3.0) as f32;
            let color = kwarg_color(&kwargs, "color", [1.0, 1.0, 1.0, 1.0]);

            // Parse cap style
            let cap_style: u8 = match kwargs.get("cap") {
                Some(Value::String(s)) if s == "square" => 2,
                Some(Value::String(s)) if s == "flat" => 0,
                _ => 1, // default round
            };

            // Animation kwargs
            let animate_plot = kwarg_bool(&kwargs, "animate", false);
            let anim_dur = kwarg_num(&kwargs, "anim_duration", 2.0);
            let anim_easing_name = kwarg_str(&kwargs, "anim_easing", "ease_out_cubic");
            let anim_easing = easing::Easing::from_str(anim_easing_name);

            let mut r = ren.borrow_mut();
            let (w, h) = (r.width() as f64, r.height() as f64);

            // Escala matemática → pixel
            let math_w = x_max - x_min;
            let scale = (w as f64 / math_w).min(h as f64 / (math_w * 0.75)) * 0.85;

            let step = (x_max - x_min) / (samples - 1) as f64;

            let mut prev_valid = false;
            let mut prev_px = 0.0f32;
            let mut prev_py = 0.0f32;
            let mut segment_count = 0usize;

            for i in 0..samples {
                let x = x_min + i as f64 * step;
                let y = match math_eval::evaluate(&expr_str, x) {
                    Ok(v) => v,
                    Err(_) => {
                        prev_valid = false;
                        continue;
                    }
                };

                if !y.is_finite() {
                    prev_valid = false;
                    continue;
                }

                let px = (x * scale) as f32;
                let py = -(y * scale) as f32;

                if prev_valid {
                    let id = r.spawn_2d_line(prev_px, prev_py, px, py, thickness);
                    let _ = r.set_stroke(id, color, thickness);
                    let _ = r.set_line_cap(id, cap_style);

                    if animate_plot {
                        let nid = id.0 as u64;
                        // Store original endpoints so animation can interpolate
                        ld.borrow_mut().insert(nid, LineData {
                            x1: prev_px, y1: prev_py,
                            x2: px, y2: py,
                        });
                        // Stagger: each segment starts after the previous one finishes
                        let delay = segment_count as f64 * (anim_dur / samples as f64);
                        an.borrow_mut().push(AnimatingLine {
                            node_id: nid,
                            duration: anim_dur / samples as f64 * 1.5, // slight overlap for smoothness
                            delay,
                            start_time: None,
                            easing: anim_easing,
                            was_hidden: true,
                        });
                        // Hide segment — the animation system will reveal it when progress > 0
                        let _ = r.set_visible(id, false);
                    }

                    segment_count += 1;
                }

                prev_px = px;
                prev_py = py;
                prev_valid = true;
            }

            Ok(Value::Number(0.0))
        })),
    );
}
