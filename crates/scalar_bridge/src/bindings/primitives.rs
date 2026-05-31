//! # Math primitive bindings for Scalar
//!
//! ## Axes
//! `Axes(x_min, x_max, y_min, y_max [, kwargs...])`
//!
//! Draws cartesian axes with ticks, optional grid, and arrows.
//! Coordinates are mathematical (not pixels) — viewport scaling is automatic.
//!
//! Full keyword argument reference:
//!
//! | Kwarg | Type | Default | Description |
//! |-------|------|---------|-------------|
//! | `grid` | Boolean | `false` | Show grid lines |
//! | `grid_color` | [r,g,b,a] | `[0.2, 0.2, 0.25, 1.0]` | Grid line color |
//! | `grid_width` | Number | `1.0` | Grid line thickness |
//! | `grid_alpha` | Number | `1.0` | Grid opacity multiplier (0.0–1.0) |
//! | `tick_step` | Number | auto | Step between major tick marks |
//! | `tick_len` | Number | `6.0` | Tick mark length in pixels |
//! | `tick_width` | Number | `1.5` | Tick mark line thickness |
//! | `tick_direction` | String | `"both"` | Tick direction: `"both"`, `"outward"`, `"inward"`, `"none"` |
//! | `minor_ticks` | Number | `0` | Number of minor ticks between major ticks |
//! | `axis_color` | [r,g,b,a] | `[0.5, 0.5, 0.5, 1.0]` | Axis line color |
//! | `x_axis_color` | [r,g,b,a] | — | Override axis color for X-axis only |
//! | `y_axis_color` | [r,g,b,a] | — | Override axis color for Y-axis only |
//! | `axis_width` | Number | `2.0` | Axis line thickness |
//! | `show_x` | Boolean | `true` | Show X-axis line, ticks, and arrow |
//! | `show_y` | Boolean | `true` | Show Y-axis line, ticks, and arrow |
//! | `arrows` | Boolean | `true` | Show arrowheads at axis ends |
//! | `arrow_size` | Number | `1.0` | Arrowhead size multiplier |
//! | `aspect` | String | `"fit"` | Aspect policy: `"fit"` (maintains ratio) or `"stretch"` |
//! | `origin` | String | `"zero"` | Where axes cross: `"zero"` (at 0,0) or `"min"` (at x_min,y_min) |
//! | `margin` | Number | `0.0` | Margin around the plot area in pixels |
//! | `x_padding` | Number | `0.0` | Fractional padding added to x-range (e.g. 0.05 = 5%) |
//! | `y_padding` | Number | `0.0` | Fractional padding added to y-range |
//! | `z_index` | Number | `0` | Base z-order for all axes elements (higher = on top) |
//! | `animate` | Boolean | `false` | Enable draw-in animation |
//! | `anim_duration` | Number | `1.5` | Animation duration in seconds |
//! | `anim_easing` | String | `"ease_out_cubic"` | Easing function name |
//!
//! ## Plot
//! `Plot("expression", x_min, x_max [, samples:, thickness:, color:, cap:, animate:, anim_duration:, anim_delay:, anim_overlap:, anim_easing:])`
//!
//! Evaluates and draws a mathematical function `f(x)` sampled over `[x_min, x_max]`.
//!
//! Animation timing is controlled by three kwargs:
//! - `anim_duration` — total time from first segment start to last segment end (default 2.0)
//! - `anim_delay` — delay before the plot starts animating (default 0.0)
//! - `anim_overlap` — overlap between consecutive segments, 0.0 = sequential, 1.0 = parallel (default 0.5)

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

/// Resolves a color from kwargs ("color:", "axis_color:", etc.) or returns default.
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

/// Resolves a number from kwargs.
fn kwarg_num(kwargs: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
    match kwargs.get(key) {
        Some(Value::Number(n)) => *n,
        _ => default,
    }
}

/// Resolves a boolean from kwargs.
fn kwarg_bool(kwargs: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    match kwargs.get(key) {
        Some(Value::Boolean(b)) => *b,
        _ => default,
    }
}

/// Resolves a string from kwargs.
fn kwarg_str<'a>(kwargs: &'a HashMap<String, Value>, key: &str, default: &'a str) -> &'a str {
    match kwargs.get(key) {
        Some(Value::String(s)) => s.as_str(),
        _ => default,
    }
}

/// Spawns a 2D line, optionally registers it for animation (hidden until revealed),
/// and sets the given z-index.
fn spawn_line_animated(
    r: &mut Renderer,
    x1: f32, y1: f32, x2: f32, y2: f32,
    width: f32, color: [f32; 4],
    animate: bool,
    line_data: &Rc<RefCell<HashMap<u64, LineData>>>,
    animations: &Rc<RefCell<Vec<AnimatingLine>>>,
    anim_dur: f64,
    anim_easing: easing::Easing,
    z_index: i32,
) {
    let id = r.spawn_2d_line(x1, y1, x2, y2, width);
    let _ = r.set_stroke(id, color, width);
    let _ = r.set_z_index(id, z_index);
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

#[allow(clippy::too_many_arguments)]
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

            // ── Parse all kwargs ────────────────────────────────────────────

            // Grid
            let grid = kwarg_bool(&kwargs, "grid", false);
            let grid_color = kwarg_color(&kwargs, "grid_color", [0.2, 0.2, 0.25, 1.0]);
            let grid_width = kwarg_num(&kwargs, "grid_width", 1.0) as f32;
            let grid_alpha = kwarg_num(&kwargs, "grid_alpha", 1.0).clamp(0.0, 1.0) as f32;

            // Ticks
            let tick_step = kwarg_num(&kwargs, "tick_step", 0.0);
            let tick_len = kwarg_num(&kwargs, "tick_len", 6.0) as f32;
            let tick_width = kwarg_num(&kwargs, "tick_width", 1.5) as f32;
            let tick_direction = kwarg_str(&kwargs, "tick_direction", "both");
            let minor_ticks = kwarg_num(&kwargs, "minor_ticks", 0.0) as i32;

            // Axes
            let axis_color = kwarg_color(&kwargs, "axis_color", [0.5, 0.5, 0.5, 1.0]);
            let axis_width = kwarg_num(&kwargs, "axis_width", 2.0) as f32;
            let show_x = kwarg_bool(&kwargs, "show_x", true);
            let show_y = kwarg_bool(&kwargs, "show_y", true);
            let x_axis_color = kwarg_color(&kwargs, "x_axis_color", axis_color);
            let y_axis_color = kwarg_color(&kwargs, "y_axis_color", axis_color);

            // Layout
            let aspect = kwarg_str(&kwargs, "aspect", "fit").to_string();
            let origin_mode = kwarg_str(&kwargs, "origin", "zero");
            let margin_px = kwarg_num(&kwargs, "margin", 0.0) as f32;
            let x_padding = kwarg_num(&kwargs, "x_padding", 0.0) as f32;
            let y_padding = kwarg_num(&kwargs, "y_padding", 0.0) as f32;

            // Arrows
            let arrows = kwarg_bool(&kwargs, "arrows", true);
            let arrow_size = kwarg_num(&kwargs, "arrow_size", 1.0) as f32;

            // Z-order
            let base_z = kwarg_num(&kwargs, "z_index", 0.0) as i32;

            // Animation
            let animate_axes = kwarg_bool(&kwargs, "animate", false);
            let anim_dur = kwarg_num(&kwargs, "anim_duration", 1.5);
            let anim_easing_name = kwarg_str(&kwargs, "anim_easing", "ease_out_cubic");
            let anim_easing = easing::Easing::from_str(anim_easing_name);

            let mut r = ren.borrow_mut();
            let (w, h) = (r.width() as f32, r.height() as f32);

            // ── Apply padding to the math range ────────────────────────────
            let x_range = x_max - x_min;
            let y_range = y_max - y_min;
            if x_range <= 0.0 || y_range <= 0.0 {
                return Err("Axes: x_max must be > x_min and y_max > y_min".to_string());
            }

            let px_min = x_min - x_range * x_padding;
            let px_max = x_max + x_range * x_padding;
            let py_min = y_min - y_range * y_padding;
            let py_max = y_max + y_range * y_padding;
            let p_range_w = px_max - px_min;
            let p_range_h = py_max - py_min;

            // ── Calculate scale with margin ────────────────────────────────
            let draw_w = w - 2.0 * margin_px;
            let draw_h = h - 2.0 * margin_px;
            if draw_w <= 0.0 || draw_h <= 0.0 {
                return Err("Axes: margin too large for viewport".to_string());
            }

            let (scale_x, scale_y) = if aspect == "stretch" {
                (draw_w / p_range_w, draw_h / p_range_h)
            } else {
                // "fit" — maintain aspect ratio, add 10% margin within the draw area
                let s = (draw_w / p_range_w).min(draw_h / p_range_h) * 0.9;
                (s, s)
            };

            // Transform math → pixel (centered in the full viewport, not just draw area)
            let origin_x = 0.0;  // center of screen (Pure2D coordinate system)
            let origin_y = 0.0;

            let to_pixel = |mx: f32, my: f32| -> (f32, f32) {
                (
                    origin_x + mx * scale_x,
                    origin_y - my * scale_y,  // Y inverted on screen
                )
            };

            // Determine where axes cross
            let axis_y = if origin_mode == "min" { py_min } else { 0.0 };
            let axis_x = if origin_mode == "min" { px_min } else { 0.0 };

            // Compute alpha-modified grid color
            let grid_color_adj = [grid_color[0], grid_color[1], grid_color[2], grid_color[3] * grid_alpha];

            // Auto tick step
            let step = if tick_step > 0.0 { tick_step as f32 } else {
                auto_tick_step(p_range_w.max(p_range_h))
            };

            // ═══════════════════════════════════════════════════════════════
            //  GRID (behind everything)
            // ═══════════════════════════════════════════════════════════════
            if grid {
                // Grid vertical
                let mut gx = (px_min / step).ceil() * step;
                while gx <= px_max {
                    if gx.abs() > step * 0.01 { // skip near origin
                        let (px, _) = to_pixel(gx, 0.0);
                        let (_, py0) = to_pixel(gx, py_min);
                        let (_, py1) = to_pixel(gx, py_max);
                        spawn_line_animated(&mut r, px, py0, px, py1, grid_width, grid_color_adj,
                            animate_axes, &ld, &an, anim_dur, anim_easing, base_z);
                    }
                    gx += step;
                }
                // Grid horizontal
                let mut gy = (py_min / step).ceil() * step;
                while gy <= py_max {
                    if gy.abs() > step * 0.01 {
                        let (_, py) = to_pixel(0.0, gy);
                        let (px0, _) = to_pixel(px_min, gy);
                        let (px1, _) = to_pixel(px_max, gy);
                        spawn_line_animated(&mut r, px0, py, px1, py, grid_width, grid_color_adj,
                            animate_axes, &ld, &an, anim_dur, anim_easing, base_z);
                    }
                    gy += step;
                }

                // ── Minor grid (if minor_ticks > 0) ──
                if minor_ticks > 0 {
                    let minor_step = step / (minor_ticks + 1) as f32;
                    let minor_color = [grid_color_adj[0], grid_color_adj[1], grid_color_adj[2], grid_color_adj[3] * 0.5];
                    let minor_width = (grid_width * 0.5).max(0.5);

                    // Minor vertical
                    let mut mgx = (px_min / minor_step).ceil() * minor_step;
                    while mgx <= px_max {
                        // Only if not near a major tick or origin
                        let dist_to_major = (mgx % step).abs().min((mgx % step - step).abs());
                        if dist_to_major > minor_step * 0.1 && mgx.abs() > step * 0.01 {
                            let (px, _) = to_pixel(mgx, 0.0);
                            let (_, py0) = to_pixel(mgx, py_min);
                            let (_, py1) = to_pixel(mgx, py_max);
                            spawn_line_animated(&mut r, px, py0, px, py1, minor_width, minor_color,
                                animate_axes, &ld, &an, anim_dur, anim_easing, base_z);
                        }
                        mgx += minor_step;
                    }
                    // Minor horizontal
                    let mut mgy = (py_min / minor_step).ceil() * minor_step;
                    while mgy <= py_max {
                        let dist_to_major = (mgy % step).abs().min((mgy % step - step).abs());
                        if dist_to_major > minor_step * 0.1 && mgy.abs() > step * 0.01 {
                            let (_, py) = to_pixel(0.0, mgy);
                            let (px0, _) = to_pixel(px_min, mgy);
                            let (px1, _) = to_pixel(px_max, mgy);
                            spawn_line_animated(&mut r, px0, py, px1, py, minor_width, minor_color,
                                animate_axes, &ld, &an, anim_dur, anim_easing, base_z);
                        }
                        mgy += minor_step;
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════
            //  X-AXIS
            // ═══════════════════════════════════════════════════════════════
            if show_x && axis_y >= py_min && axis_y <= py_max {
                let x_axis_z = base_z + 1;
                let (px0, py) = to_pixel(px_min, axis_y);
                let (px1, _) = to_pixel(px_max, axis_y);
                spawn_line_animated(&mut r, px0, py, px1, py, axis_width, x_axis_color,
                    animate_axes, &ld, &an, anim_dur, anim_easing, x_axis_z);

                // ── X ticks ──
                if tick_direction != "none" {
                    let (_, y_axis_px) = to_pixel(0.0, axis_y);
                    let mut tx = (px_min / step).ceil() * step;
                    while tx <= px_max {
                        if tx.abs() > step * 0.01 {
                            let (px, _) = to_pixel(tx, 0.0);
                            let (y_top, y_bot) = match tick_direction {
                                "outward" => (y_axis_px, y_axis_px + tick_len),
                                "inward"  => (y_axis_px - tick_len, y_axis_px),
                                _ /* both */ => (y_axis_px - tick_len * 0.5, y_axis_px + tick_len * 0.5),
                            };
                            spawn_line_animated(&mut r, px, y_top, px, y_bot, tick_width, x_axis_color,
                                animate_axes, &ld, &an, anim_dur, anim_easing, x_axis_z + 1);
                        }
                        tx += step;
                    }

                    // ── Minor X ticks ──
                    if minor_ticks > 0 {
                        let minor_step = step / (minor_ticks + 1) as f32;
                        let minor_tick_len = tick_len * 0.5;
                        let mut mtx = (px_min / minor_step).ceil() * minor_step;
                        while mtx <= px_max {
                            let dist_to_major = (mtx % step).abs().min((mtx % step - step).abs());
                            if dist_to_major > minor_step * 0.1 && mtx.abs() > step * 0.01 {
                                let (px, _) = to_pixel(mtx, 0.0);
                                let (y_top, y_bot) = match tick_direction {
                                    "outward" => (y_axis_px, y_axis_px + minor_tick_len),
                                    "inward"  => (y_axis_px - minor_tick_len, y_axis_px),
                                    _ /* both */ => (y_axis_px - minor_tick_len * 0.5, y_axis_px + minor_tick_len * 0.5),
                                };
                                spawn_line_animated(&mut r, px, y_top, px, y_bot, tick_width * 0.5, x_axis_color,
                                    animate_axes, &ld, &an, anim_dur, anim_easing, x_axis_z + 1);
                            }
                            mtx += minor_step;
                        }
                    }
                }

                // ── X arrow ──
                if arrows {
                    let (tip_x, tip_y) = to_pixel(px_max, axis_y);
                    let (base_x, base_y) = to_pixel(px_max - p_range_w * 0.02, axis_y);
                    draw_arrow_animated(&mut r, tip_x, tip_y, base_x, base_y, x_axis_color, axis_width,
                        animate_axes, &ld, &an, anim_dur, anim_easing, x_axis_z + 2, arrow_size);
                }
            }

            // ═══════════════════════════════════════════════════════════════
            //  Y-AXIS
            // ═══════════════════════════════════════════════════════════════
            if show_y && axis_x >= px_min && axis_x <= px_max {
                let y_axis_z = base_z + 1;
                let (px, py0) = to_pixel(axis_x, py_min);
                let (_, py1) = to_pixel(axis_x, py_max);
                spawn_line_animated(&mut r, px, py0, px, py1, axis_width, y_axis_color,
                    animate_axes, &ld, &an, anim_dur, anim_easing, y_axis_z);

                // ── Y ticks ──
                if tick_direction != "none" {
                    let (x_axis_px, _) = to_pixel(axis_x, 0.0);
                    let mut ty = (py_min / step).ceil() * step;
                    while ty <= py_max {
                        if ty.abs() > step * 0.01 {
                            let (_, py) = to_pixel(0.0, ty);
                            let (x_left, x_right) = match tick_direction {
                                "outward" => (x_axis_px - tick_len, x_axis_px),
                                "inward"  => (x_axis_px, x_axis_px + tick_len),
                                _ /* both */ => (x_axis_px - tick_len * 0.5, x_axis_px + tick_len * 0.5),
                            };
                            spawn_line_animated(&mut r, x_left, py, x_right, py, tick_width, y_axis_color,
                                animate_axes, &ld, &an, anim_dur, anim_easing, y_axis_z + 1);
                        }
                        ty += step;
                    }

                    // ── Minor Y ticks ──
                    if minor_ticks > 0 {
                        let minor_step = step / (minor_ticks + 1) as f32;
                        let minor_tick_len = tick_len * 0.5;
                        let mut mty = (py_min / minor_step).ceil() * minor_step;
                        while mty <= py_max {
                            let dist_to_major = (mty % step).abs().min((mty % step - step).abs());
                            if dist_to_major > minor_step * 0.1 && mty.abs() > step * 0.01 {
                                let (_, py) = to_pixel(0.0, mty);
                                let (x_left, x_right) = match tick_direction {
                                    "outward" => (x_axis_px - minor_tick_len, x_axis_px),
                                    "inward"  => (x_axis_px, x_axis_px + minor_tick_len),
                                    _ /* both */ => (x_axis_px - minor_tick_len * 0.5, x_axis_px + minor_tick_len * 0.5),
                                };
                                spawn_line_animated(&mut r, x_left, py, x_right, py, tick_width * 0.5, y_axis_color,
                                    animate_axes, &ld, &an, anim_dur, anim_easing, y_axis_z + 1);
                            }
                            mty += minor_step;
                        }
                    }
                }

                // ── Y arrow ──
                if arrows {
                    let (tip_x, tip_y) = to_pixel(axis_x, py_max);
                    let (base_x, base_y) = to_pixel(axis_x, py_max - p_range_h * 0.02);
                    draw_arrow_animated(&mut r, tip_x, tip_y, base_x, base_y, y_axis_color, axis_width,
                        animate_axes, &ld, &an, anim_dur, anim_easing, y_axis_z + 2, arrow_size);
                }
            }

            Ok(Value::Number(0.0))
        })),
    );
}

/// Draws an arrowhead (2 angled lines) with optional animation support and size multiplier.
#[allow(clippy::too_many_arguments)]
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
    z_index: i32,
    arrow_size: f32,
) {
    let dx = tx - bx;
    let dy = ty - by;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 { return; }
    let nx = dx / len;
    let ny = dy / len;
    let spread = (8.0 + width * 2.0) * arrow_size;

    // Left barb
    let lx = tx - nx * spread + ny * spread * 0.4;
    let ly = ty - ny * spread - nx * spread * 0.4;
    spawn_line_animated(r, tx, ty, lx, ly, width.max(1.5), color,
        animate, line_data, animations, anim_dur, anim_easing, z_index);

    // Right barb
    let rx = tx - nx * spread - ny * spread * 0.4;
    let ry = ty - ny * spread + nx * spread * 0.4;
    spawn_line_animated(r, tx, ty, rx, ry, width.max(1.5), color,
        animate, line_data, animations, anim_dur, anim_easing, z_index);
}

/// Picks a reasonable tick step for a given range.
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
            let anim_delay = kwarg_num(&kwargs, "anim_delay", 0.0);
            let anim_overlap = kwarg_num(&kwargs, "anim_overlap", 0.5).clamp(0.0, 1.0);
            let anim_easing_name = kwarg_str(&kwargs, "anim_easing", "ease_out_cubic");
            let anim_easing = easing::Easing::from_str(anim_easing_name);

            let mut r = ren.borrow_mut();
            let (w, h) = (r.width() as f64, r.height() as f64);

            // Math → pixel scale
            let math_w = x_max - x_min;
            let scale = (w as f64 / math_w).min(h as f64 / (math_w * 0.75)) * 0.85;

            let step = (x_max - x_min) / (samples - 1) as f64;

            let mut prev_valid = false;
            let mut prev_px = 0.0f32;
            let mut prev_py = 0.0f32;
            let mut segments: Vec<LineData> = Vec::new();

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
                    segments.push(LineData {
                        x1: prev_px, y1: prev_py,
                        x2: px, y2: py,
                    });
                }

                prev_px = px;
                prev_py = py;
                prev_valid = true;
            }

            // ── Spawn all segments (as lines), then register animation if enabled ──
            let n_segments = segments.len();
            for (seg_idx, seg) in segments.iter().enumerate() {
                let id = r.spawn_2d_line(seg.x1, seg.y1, seg.x2, seg.y2, thickness);
                let _ = r.set_stroke(id, color, thickness);
                let _ = r.set_line_cap(id, cap_style);

                if animate_plot && n_segments > 0 {
                    let nid = id.0 as u64;
                    // Store original endpoints so animation can interpolate
                    ld.borrow_mut().insert(nid, seg.clone());

                    // Compute segment timing:
                    //   total_dur = anim_dur  (time from first segment start to last segment end)
                    //   overlap ∈ [0, 1] where 0 = sequential, 1 = fully parallel
                    //   segment_duration = total_dur / (1 + (n-1) * (1-overlap))
                    //   delay_between    = segment_duration * (1-overlap)
                    let factor = 1.0 + (n_segments as f64 - 1.0) * (1.0 - anim_overlap);
                    let seg_duration = if factor > 0.0 { anim_dur / factor } else { anim_dur };
                    let delay_between = seg_duration * (1.0 - anim_overlap);

                    an.borrow_mut().push(AnimatingLine {
                        node_id: nid,
                        duration: seg_duration,
                        delay: anim_delay + seg_idx as f64 * delay_between,
                        start_time: None,
                        easing: anim_easing,
                        was_hidden: true,
                    });
                    // Hide segment — the animation system will reveal it when progress > 0
                    let _ = r.set_visible(id, false);
                }
            }

            Ok(Value::Number(0.0))
        })),
    );
}
