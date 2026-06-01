use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::{Renderer, NodeId};
use crate::{LineData, AnimationEntry, AnimationKind};
use crate::bindings::imports::{FontEntry, glyph_paths_for_text};
use crate::bindings::shapes::{self, num, kwarg_num};

pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
    fonts: Rc<RefCell<Vec<FontEntry>>>,
) {
    register_set_line_progress(env, renderer.clone(), line_data.clone());
    register_set_line_cap(env, renderer.clone());
    register_animate(env, renderer.clone(), line_data.clone(), animations.clone());
    register_fade_in(env, animations.clone());
    register_fade_out(env, animations.clone());
    register_grow(env, animations.clone());
    register_shrink(env, animations.clone());
    register_move_to(env, animations.clone());
    register_draw_then_fill(env, animations.clone());
    register_write_text(env, renderer.clone(), animations.clone(), fonts.clone());
    register_reveal_text(env, renderer.clone(), animations.clone(), fonts);
}

/// Extracts a node ID from an argument (NodeId or Number).
fn arg_to_node_id(args: &[Value], index: usize) -> Result<u64, String> {
    match args.get(index) {
        Some(Value::NodeId(id)) => Ok(*id as u64),
        Some(Value::Number(n)) => Ok(*n as u64),
        Some(_) => Err("argument must be a NodeId or Number".to_string()),
        None => Err("not enough arguments".to_string()),
    }
}

/// Extracts a list of node IDs from kwargs `lines:`, from a List first positional arg,
/// or from a single NodeId/Number positional arg.
fn parse_node_ids(args: &[Value], kwargs: &HashMap<String, Value>) -> Result<Vec<u64>, String> {
    // 1. Check `lines:` kwarg
    if let Some(Value::List(list)) = kwargs.get("lines") {
        let ids: Vec<u64> = list
            .iter()
            .filter_map(|v| match v {
                Value::NodeId(id) => Some(*id as u64),
                Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .collect();
        if ids.is_empty() {
            return Err("no valid node IDs in `lines:` list".to_string());
        }
        return Ok(ids);
    }

    // 2. Check if first positional arg is a List of node IDs (e.g. from WriteText)
    if let Some(Value::List(list)) = args.get(0) {
        let ids: Vec<u64> = list
            .iter()
            .filter_map(|v| match v {
                Value::NodeId(id) => Some(*id as u64),
                Value::Number(n) => Some(*n as u64),
                _ => None,
            })
            .collect();
        if !ids.is_empty() {
            return Ok(ids);
        }
    }

    // 3. Single node ID from first positional arg
    let id = arg_to_node_id(args, 0)?;
    Ok(vec![id])
}

/// Parse animation meta-kwargs common to all animation functions.
struct AnimParams {
    duration: f64,
    delay: f64,
    easing: crate::easing::Easing,
}

fn parse_anim_params(kwargs: &HashMap<String, Value>) -> AnimParams {
    let duration = kwarg_num(kwargs, "duration", 1.0);
    let delay = kwarg_num(kwargs, "delay", 0.0);
    let easing = match kwargs.get("easing") {
        Some(Value::String(s)) => crate::easing::Easing::from_str(s),
        _ => crate::easing::Easing::EaseOutCubic,
    };
    AnimParams { duration, delay, easing }
}

// ─── SetLineCap ───────────────────────────────────────────────────────────────

/// SetLineCap(node_id, cap)
///
/// Changes the line cap style of an existing line.
/// `cap` can be: "flat" (0), "round" (1), "square" (2).
fn register_set_line_cap(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "SetLineCap".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            if args.len() < 2 {
                return Err("SetLineCap(node_id, cap) requires 2 arguments".to_string());
            }
            let node_id = match &args[0] {
                Value::NodeId(id) => *id as u64,
                Value::Number(n) => *n as u64,
                _ => return Err("SetLineCap: first argument must be a NodeId".to_string()),
            };
            let cap: u8 = match &args[1] {
                Value::String(s) if s == "round" => 1,
                Value::String(s) if s == "square" => 2,
                Value::String(s) if s == "flat" => 0,
                Value::Number(n) => n.clamp(0.0, 2.0) as u8,
                _ => return Err("SetLineCap: cap must be \"flat\", \"round\", \"square\", or a number 0-2".to_string()),
            };

            let mut r = ren.borrow_mut();
            let _ = r.set_line_cap(NodeId(node_id), cap);
            Ok(Value::Number(0.0))
        })),
    );
}

// ─── SetLineProgress ──────────────────────────────────────────────────────────

/// SetLineProgress(node_id, progress)
///
/// Interpolates the line endpoint so that `progress=0.0` collapses the line
/// at its start point and `progress=1.0` draws the full segment.
fn register_set_line_progress(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
) {
    let ren = renderer.clone();
    let ld = line_data.clone();
    env.define(
        "SetLineProgress".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            if args.len() < 2 {
                return Err("SetLineProgress(node_id, progress) requires 2 arguments".to_string());
            }
            let node_id = match &args[0] {
                Value::NodeId(id) => *id as u64,
                Value::Number(n) => *n as u64,
                _ => return Err("SetLineProgress: first argument must be a NodeId".to_string()),
            };
            let progress = match &args[1] {
                Value::Number(n) => n.clamp(0.0, 1.0) as f32,
                _ => return Err("SetLineProgress: progress must be a number (0.0–1.0)".to_string()),
            };

            let data = ld.borrow();
            if let Some(line) = data.get(&node_id) {
                let end_x = line.x1 + (line.x2 - line.x1) * progress;
                let end_y = line.y1 + (line.y2 - line.y1) * progress;
                let mut r = ren.borrow_mut();
                let _ = r.update_line_endpoints(NodeId(node_id), line.x1, line.y1, end_x, end_y);
            }
            Ok(Value::Number(0.0))
        })),
    );
}

// ─── Animate (line draw) ──────────────────────────────────────────────────────

/// Animate(id, duration: 1.0)
/// Animate(lines: [id1, id2, ...], per_line: 1.0, staggered: true)
///
/// Registers a line-draw animation. Each line draws from start to end over
/// the specified duration. When `staggered` is true (default), lines animate
/// sequentially — each one starts after the previous one finishes.
/// When `staggered` is false, all lines animate in parallel.
fn register_animate(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let ren = renderer.clone();
    let ld = line_data.clone();
    let an = animations.clone();
    env.define(
        "Animate".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids: Vec<u64> = if let Some(Value::List(list)) = kwargs.get("lines") {
                list.iter().filter_map(|v| match v {
                    Value::NodeId(id) => Some(*id as u64),
                    Value::Number(n) => Some(*n as u64),
                    _ => None,
                }).collect()
            } else if let Some(Value::NodeId(id)) = args.get(0) {
                vec![*id as u64]
            } else if let Some(Value::Number(n)) = args.get(0) {
                vec![*n as u64]
            } else {
                return Err("Animate: first argument must be a NodeId or use lines: [id1, id2, ...]".to_string());
            };

            if node_ids.is_empty() {
                return Err("Animate: no valid node IDs provided".to_string());
            }

            let staggered = match kwargs.get("staggered") {
                Some(Value::Boolean(b)) => *b,
                _ => true,
            };

            let easing_name = match kwargs.get("easing") {
                Some(Value::String(s)) => crate::easing::Easing::from_str(s),
                _ => crate::easing::Easing::EaseOutCubic,
            };

            let per_line_dur = match kwargs.get("per_line") {
                Some(Value::Number(n)) => *n,
                _ => match kwargs.get("duration") {
                    Some(Value::Number(n)) => *n / node_ids.len() as f64,
                    _ => 1.0,
                },
            };

            // Hide all lines before animation begins
            {
                let mut r = ren.borrow_mut();
                for nid in &node_ids {
                    if !ld.borrow().contains_key(nid) {
                        return Err(format!("Animate: node {} is not a tracked line", nid));
                    }
                    let _ = r.set_visible(NodeId(*nid), false);
                }
            }

            let mut anims = an.borrow_mut();
            for (i, nid) in node_ids.iter().enumerate() {
                let delay = if staggered {
                    i as f64 * per_line_dur
                } else {
                    0.0
                };
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: per_line_dur,
                    delay,
                    start_time: None,
                    easing: easing_name,
                    kind: AnimationKind::LineDraw,
                    was_hidden: true,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── FadeIn ───────────────────────────────────────────────────────────────────

/// FadeIn(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates a node's opacity from 0.0 → its current opacity (or 1.0 if fully opaque).
fn register_fade_in(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "FadeIn".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids = parse_node_ids(&args, &kwargs)?;
            let ap = parse_anim_params(&kwargs);

            let mut anims = an.borrow_mut();
            for (i, nid) in node_ids.iter().enumerate() {
                let delay = ap.delay + i as f64 * 0.0; // all fade in parallel by default
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: ap.duration,
                    delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::Fade {
                        from_opacity: 0.0,
                        to_opacity: 1.0,
                    },
                    was_hidden: false,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── FadeOut ──────────────────────────────────────────────────────────────────

/// FadeOut(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates a node's opacity from its current value → 0.0.
fn register_fade_out(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "FadeOut".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids = parse_node_ids(&args, &kwargs)?;
            let ap = parse_anim_params(&kwargs);

            let mut anims = an.borrow_mut();
            for (i, nid) in node_ids.iter().enumerate() {
                let delay = ap.delay + i as f64 * 0.0;
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: ap.duration,
                    delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::Fade {
                        from_opacity: 1.0,
                        to_opacity: 0.0,
                    },
                    was_hidden: false,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── Grow ─────────────────────────────────────────────────────────────────────

/// Grow(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates a node's uniform scale from 0.0 → its current scale.
fn register_grow(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "Grow".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids = parse_node_ids(&args, &kwargs)?;
            let ap = parse_anim_params(&kwargs);

            let mut anims = an.borrow_mut();
            for nid in &node_ids {
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: ap.duration,
                    delay: ap.delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::Scale {
                        from: None,  // captured on first frame
                        to: 1.0,
                    },
                    was_hidden: false,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── Shrink ───────────────────────────────────────────────────────────────────

/// Shrink(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates a node's uniform scale from its current value → 0.0.
fn register_shrink(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "Shrink".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids = parse_node_ids(&args, &kwargs)?;
            let ap = parse_anim_params(&kwargs);

            let mut anims = an.borrow_mut();
            for nid in &node_ids {
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: ap.duration,
                    delay: ap.delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::Scale {
                        from: None,  // captured on first frame
                        to: 0.0,
                    },
                    was_hidden: false,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── MoveTo ───────────────────────────────────────────────────────────────────

/// MoveTo(node_id, x, y, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates a node's position from its current position → (x, y).
fn register_move_to(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "MoveTo".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            // Parse node_id from first positional arg
            let node_id = arg_to_node_id(&args, 0)?;
            // Parse target x, y from positional args 1 and 2 (or kwargs)
            let to_x = if args.len() > 1 {
                num(args.get(1)) as f32
            } else {
                kwarg_num(&kwargs, "x", 0.0) as f32
            };
            let to_y = if args.len() > 2 {
                num(args.get(2)) as f32
            } else {
                kwarg_num(&kwargs, "y", 0.0) as f32
            };
            let ap = parse_anim_params(&kwargs);

            let mut anims = an.borrow_mut();
            anims.push(AnimationEntry {
                node_id,
                duration: ap.duration,
                delay: ap.delay,
                start_time: None,
                easing: ap.easing,
                kind: AnimationKind::MoveTo {
                    from_x: None,  // captured on first frame
                    from_y: None,
                    to_x,
                    to_y,
                },
                was_hidden: false,
            });

            Ok(Value::Number(1.0))
        })),
    );
}

// ─── DrawThenFill ─────────────────────────────────────────────────────────────

/// DrawThenFill(node_id, duration: 1.0, delay: 0.0, easing: "ease_out_cubic",
///              fill: [r,g,b,a])
///
/// Two-phase reveal animation similar to Manim's `DrawBorderThenFill`:
/// - Phase 1 (0–60% of eased progress): node scales from 0→1, fill transparent
/// - Phase 2 (60–100%): scale stays at 1, fill fades from transparent → original
///
/// Kwargs:
/// | Kwarg | Type | Default | Description |
/// |-------|------|---------|-------------|
/// | `fill` | [r,g,b,a] | `[1,1,1,1]` | Target fill color to animate in |
/// | `duration` | Number | `1.0` | Duration |
/// | `delay` | Number | `0.0` | Delay before start |
/// | `easing` | String | `"ease_out_cubic"` | Easing function |
fn register_draw_then_fill(
    env: &mut Environment,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let an = animations.clone();
    env.define(
        "DrawThenFill".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let node_ids = parse_node_ids(&args, &kwargs)?;
            let ap = parse_anim_params(&kwargs);

            // Parse fill color kwarg (values are 0–1 range, same as all shape kwargs)
            let fill_color: [f32; 4] = match kwargs.get("fill") {
                Some(Value::List(vals)) if vals.len() >= 3 => [
                    num(Some(&vals[0])) as f32,
                    num(Some(&vals[1])) as f32,
                    num(Some(&vals[2])) as f32,
                    if vals.len() >= 4 { num(Some(&vals[3])) as f32 } else { 1.0 },
                ],
                _ => [1.0, 1.0, 1.0, 1.0],
            };

            let mut anims = an.borrow_mut();
            for nid in &node_ids {
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: ap.duration,
                    delay: ap.delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::DrawThenFill {
                        from_scale: None,
                        fill_rgba: fill_color,
                    },
                    was_hidden: false,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}

// ─── WriteText ────────────────────────────────────────────────────────────────

/// WriteText(str, x, y, font: 0, size: 48, duration: 1.0, per_char: ..., ...kwargs)
///
/// Renders text character-by-character with sequential fade-in animation.
/// Each character is a separate NodeId, and they appear one after another
/// from left to right, similar to Manim's `Write`.
///
/// Returns a `List` of NodeIds (one per character).
///
/// Kwargs:
/// | Kwarg | Type | Default | Description |
/// |-------|------|---------|-------------|
/// | `font` | Number | `0` | Font index from `FontImport()` |
/// | `size` | Number | `48` | Font size in pixels |
/// | `duration` | Number | `1.0` | Total animation duration (per_char takes precedence) |
/// | `per_char` | Number | — | Duration per character (overrides `duration`) |
/// | `easing` | String | `"ease_out_cubic"` | Easing function |
/// | `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
/// | `stroke` | [r,g,b,a] | — | Stroke color |
/// | `stroke_width` | Number | `2.0` | Stroke thickness |
/// | `opacity` | Number | `1.0` | Global opacity |
/// | `z_index` | Number | `0` | Z-order |
/// | `rotation` | Number | `0` | Rotation in degrees |
fn register_write_text(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
    fonts: Rc<RefCell<Vec<FontEntry>>>,
) {
    let ren = renderer.clone();
    let an = animations.clone();
    let ft = fonts.clone();
    env.define(
        "WriteText".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let text_str = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("WriteText(str, x, y, ...): first argument must be a string".to_string()),
            };

            let x = if args.len() > 1 {
                num(args.get(1)) as f32
            } else {
                kwarg_num(&kwargs, "x", 0.0) as f32
            };
            let y = if args.len() > 2 {
                num(args.get(2)) as f32
            } else {
                kwarg_num(&kwargs, "y", 0.0) as f32
            };

            let font_index = kwarg_num(&kwargs, "font", 0.0) as usize;
            let font_size = kwarg_num(&kwargs, "size", 48.0) as f32;

            let font_list = ft.borrow();
            let font_bytes = match font_list.get(font_index) {
                Some(entry) => &entry.bytes,
                None => return Err(format!(
                    "WriteText: font index {} not found. Load a font first with FontImport(path).",
                    font_index
                )),
            };

            // Build per-character path data
            let glyphs = glyph_paths_for_text(font_bytes, &text_str, font_size)?;

            // Determine animation timing
            let easing_name = match kwargs.get("easing") {
                Some(Value::String(s)) => crate::easing::Easing::from_str(s),
                _ => crate::easing::Easing::EaseOutCubic,
            };

            let per_char_dur = match kwargs.get("per_char") {
                Some(Value::Number(n)) => *n,
                _ => {
                    let total = kwarg_num(&kwargs, "duration", 1.0);
                    if glyphs.len() > 0 {
                        total / glyphs.len() as f64
                    } else {
                        total
                    }
                }
            };

            // Build kwargs for each character (skip font/size/position, pass through rest)
            let mut char_kwargs = kwargs.clone();
            char_kwargs.remove("font");
            char_kwargs.remove("size");
            char_kwargs.remove("duration");
            char_kwargs.remove("per_char");
            char_kwargs.remove("easing");
            char_kwargs.remove("x");
            char_kwargs.remove("y");

            let mut r = ren.borrow_mut();
            let mut node_ids: Vec<Value> = Vec::with_capacity(glyphs.len());

            for (i, glyph) in glyphs.iter().enumerate() {
                // NOTE: cursor_x is already baked into each glyph's path coordinates
                // by glyph_paths_for_text(). Spawning at (x, y) without adding advance
                // widths avoids double-spacing.
                let sk = shapes::parse_shape_kwargs(&char_kwargs);
                let id = shapes::spawn_2d_shape_with_kwargs(&mut r, x, y, glyph.commands.clone(), &sk);

                // Hide it initially — will be revealed by the fade-in animation
                let _ = r.set_visible(NodeId(id as u64), false);

                // Register a fade-in animation for this character
                let delay = i as f64 * per_char_dur;
                an.borrow_mut().push(AnimationEntry {
                    node_id: id as u64,
                    duration: per_char_dur,
                    delay,
                    start_time: None,
                    easing: easing_name,
                    kind: AnimationKind::Fade {
                        from_opacity: 0.0,
                        to_opacity: 1.0,
                    },
                    was_hidden: true,
                });

                node_ids.push(Value::NodeId(id));
            }

            Ok(Value::List(node_ids))
        })),
    );
}

// ─── RevealText ───────────────────────────────────────────────────────────────

/// RevealText(str, x, y, font: 0, size: 48, duration: 1.0, ...kwargs)
///
/// Renders text character-by-character with a "draw then fill" reveal animation.
/// Each character first scales from 0→1 (stroke visible, fill transparent), then
/// fill fades in from transparent → original color. Similar to Manim's
/// `DrawBorderThenFill`.
///
/// Returns a `List` of NodeIds (one per character).
///
/// Kwargs are identical to WriteText:
/// | Kwarg | Type | Default | Description |
/// |-------|------|---------|-------------|
/// | `font` | Number | `0` | Font index from `FontImport()` |
/// | `size` | Number | `48` | Font size in pixels |
/// | `duration` | Number | `1.0` | Total animation duration (per_char takes precedence) |
/// | `per_char` | Number | — | Duration per character (overrides `duration`) |
/// | `easing` | String | `"ease_out_cubic"` | Easing function |
/// | `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
/// | `stroke` | [r,g,b,a] | — | Stroke color |
/// | `stroke_width` | Number | `2.0` | Stroke thickness |
/// | `opacity` | Number | `1.0` | Global opacity |
/// | `z_index` | Number | `0` | Z-order |
/// | `rotation` | Number | `0` | Rotation in degrees |
fn register_reveal_text(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
    fonts: Rc<RefCell<Vec<FontEntry>>>,
) {
    let ren = renderer.clone();
    let an = animations.clone();
    let ft = fonts.clone();
    env.define(
        "RevealText".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let text_str = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("RevealText(str, x, y, ...): first argument must be a string".to_string()),
            };

            let x = if args.len() > 1 {
                num(args.get(1)) as f32
            } else {
                kwarg_num(&kwargs, "x", 0.0) as f32
            };
            let y = if args.len() > 2 {
                num(args.get(2)) as f32
            } else {
                kwarg_num(&kwargs, "y", 0.0) as f32
            };

            let font_index = kwarg_num(&kwargs, "font", 0.0) as usize;
            let font_size = kwarg_num(&kwargs, "size", 48.0) as f32;

            let font_list = ft.borrow();
            let font_bytes = match font_list.get(font_index) {
                Some(entry) => &entry.bytes,
                None => return Err(format!(
                    "RevealText: font index {} not found. Load a font first with FontImport(path).",
                    font_index
                )),
            };

            // Build per-character path data
            let glyphs = glyph_paths_for_text(font_bytes, &text_str, font_size)?;

            // Determine animation timing
            let easing_name = match kwargs.get("easing") {
                Some(Value::String(s)) => crate::easing::Easing::from_str(s),
                _ => crate::easing::Easing::EaseOutCubic,
            };

            let per_char_dur = match kwargs.get("per_char") {
                Some(Value::Number(n)) => *n,
                _ => {
                    let total = kwarg_num(&kwargs, "duration", 1.0);
                    if glyphs.len() > 0 {
                        total / glyphs.len() as f64
                    } else {
                        total
                    }
                }
            };

            // Build kwargs for each character (skip font/size/position, pass through rest)
            let mut char_kwargs = kwargs.clone();
            char_kwargs.remove("font");
            char_kwargs.remove("size");
            char_kwargs.remove("duration");
            char_kwargs.remove("per_char");
            char_kwargs.remove("easing");
            char_kwargs.remove("x");
            char_kwargs.remove("y");

            // Extract fill color for the DrawThenFill animation (values are 0–1 range)
            let fill_color: [f32; 4] = match kwargs.get("fill") {
                Some(Value::List(vals)) if vals.len() >= 3 => [
                    num(Some(&vals[0])) as f32,
                    num(Some(&vals[1])) as f32,
                    num(Some(&vals[2])) as f32,
                    if vals.len() >= 4 { num(Some(&vals[3])) as f32 } else { 1.0 },
                ],
                _ => [1.0, 1.0, 1.0, 1.0], // default white
            };
            // Apply global opacity to fill alpha
            let global_opacity = kwarg_num(&kwargs, "opacity", 1.0) as f32;
            let fill_rgba = [fill_color[0], fill_color[1], fill_color[2], fill_color[3] * global_opacity];

            let mut r = ren.borrow_mut();
            let mut node_ids: Vec<Value> = Vec::with_capacity(glyphs.len());

            for (i, glyph) in glyphs.iter().enumerate() {
                // cursor_x is baked into path coordinates, spawn all at (x, y)
                let sk = shapes::parse_shape_kwargs(&char_kwargs);
                let id = shapes::spawn_2d_shape_with_kwargs(&mut r, x, y, glyph.commands.clone(), &sk);

                // Hide it initially — will be revealed by the DrawThenFill animation
                let _ = r.set_visible(NodeId(id as u64), false);

                // Register a DrawThenFill animation for this character
                let delay = i as f64 * per_char_dur;
                an.borrow_mut().push(AnimationEntry {
                    node_id: id as u64,
                    duration: per_char_dur,
                    delay,
                    start_time: None,
                    easing: easing_name,
                    kind: AnimationKind::DrawThenFill {
                        from_scale: None, // lazily captured on first frame
                        fill_rgba,
                    },
                    was_hidden: true,
                });

                node_ids.push(Value::NodeId(id));
            }

            Ok(Value::List(node_ids))
        })),
    );
}
