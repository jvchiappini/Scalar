use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::{Renderer, NodeId};
use crate::{LineData, AnimationEntry, AnimationKind, sample_path_uniformly};
use crate::bindings::imports::{FontEntry, glyph_paths_for_text, extract_path_segments, subdivide_segments};
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
    register_reveal_text(env, renderer.clone(), animations.clone(), fonts.clone());
    register_morph(env, renderer.clone(), animations.clone());
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

// ─── Animate (general-purpose) ──────────────────────────────────────────────

/// Universal animation dispatcher — Manim-style professional animation.
///
/// # Signatures
///
/// ```scalar
/// // Animate a single element
/// Animate(target, "FadeIn", duration: 1.0, delay: 0.0)
/// Animate(target, "FadeOut", duration: 1.0)
/// Animate(target, "Grow", duration: 1.0)
/// Animate(target, "Shrink", duration: 1.0)
/// Animate(target, "DrawThenFill", duration: 1.0, fill: [r,g,b,a])
/// Animate(target, "MoveTo", x: 100, y: 200, duration: 1.0)
///
/// // Animate a list of elements with staggered per-glyph timing
/// Animate(parts, "FadeIn", duration: 0.25, stagger: 0.08, delay: 19.5)
/// Animate(parts, "DrawThenFill", duration: 0.4, stagger: 0.1, fill: [1,0.4,0.4,1])
///
/// // Line-draw animation (backward compat: effect defaults to "LineDraw")
/// Animate(line_id, duration: 1.0)
/// Animate(lines: [id1, id2, ...], per_line: 0.5, staggered: true)
/// ```
///
/// # Effect Reference
///
/// | Effect | Kwargs | Description |
/// |--------|--------|-------------|
/// | `FadeIn` | — | Opacity 0→1 |
/// | `FadeOut` | — | Opacity 1→0 |
/// | `Grow` | — | Scale 0→1 |
/// | `Shrink` | — | Scale 1→0 |
/// | `DrawThenFill` | `fill: [r,g,b,a]` | Phase 1: draw outline, Phase 2: fill |
/// | `MoveTo` | `x: N, y: N` | Translate to (x, y) |
/// | `LineDraw` | — | Animate line endpoint from stored LineData |
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
            // ── Parse targets (NodeId or [NodeId]) ─────────────────────
            let node_ids: Vec<u64> = if let Some(Value::List(list)) = kwargs.get("lines") {
                // Backward compat: lines: kwarg (old-style line draw)
                list.iter().filter_map(|v| match v {
                    Value::NodeId(id) => Some(*id as u64),
                    Value::Number(n) => Some(*n as u64),
                    _ => None,
                }).collect()
            } else if let Some(Value::List(list)) = args.get(0) {
                // First positional is a List of NodeIds
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
                return Err(
                    "Animate(target, \"Effect\", ...): first argument must be a \
                     NodeId or [NodeId] (or use `lines: [id, ...]` for backward compat)"
                    .to_string(),
                );
            };

            if node_ids.is_empty() {
                return Err("Animate: no valid node IDs provided".to_string());
            }

            // ── Parse effect name ──────────────────────────────────────
            // Second positional arg is the effect name (String).
            // If absent, default to "LineDraw" for backward compat.
            let effect = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => "LineDraw".to_string(),
            };

            // ── Parse standard animation kwargs ────────────────────────
            let params = parse_anim_params(&kwargs);
            let duration = params.duration;
            let delay = params.delay;
            let easing = params.easing;
            let stagger = match kwargs.get("stagger") {
                Some(Value::Number(n)) => *n,
                _ => 0.0,
            };

            // ── Backward compat for old-style line draw ────────────────
            // Old `Animate(lines: [...], per_line: N, staggered: true/false)`
            let per_line_override = match kwargs.get("per_line") {
                Some(Value::Number(n)) => Some(*n),
                _ => None,
            };
            let use_staggered = match kwargs.get("staggered") {
                Some(Value::Boolean(b)) => *b,
                _ => true,
            };

            let eff_duration = per_line_override.unwrap_or(duration);

            // ── Pre-parse source IDs for Morph effect ─────────────────
            // The `into:` kwarg can be a single NodeId or a [NodeId] (e.g. from Tex()).
            let morph_source_ids: Option<Vec<u64>> = if effect == "Morph" {
                Some(match kwargs.get("into") {
                    Some(val) => crate::value_to_ids(val).map_err(|e| {
                        format!("Animate(..., \"Morph\", into: ...): {}", e)
                    })?,
                    None => return Err(
                        "Animate(..., \"Morph\", into: source_node): \
                         'into' kwarg is required"
                        .to_string(),
                    ),
                })
            } else {
                None
            };

            // Compute per-element delay: if explicit stagger > 0 use it;
            // otherwise for old-style use per_line or duration/count.
            let per_item_delay = |i: usize| -> f64 {
                if stagger > 0.0 {
                    delay + i as f64 * stagger
                } else if effect == "LineDraw" && (per_line_override.is_some() || use_staggered) {
                    if use_staggered { i as f64 * eff_duration } else { 0.0 }
                } else {
                    delay
                }
            };

            // ── Validate LineDraw targets ──────────────────────────────
            if effect == "LineDraw" {
                let ld_map = ld.borrow();
                for nid in &node_ids {
                    if !ld_map.contains_key(nid) {
                        return Err(format!(
                            "Animate(..., \"LineDraw\"): node {} is not a tracked line. \
                             Lines must be created with `Line(x1, y1, x2, y2, ...)`",
                            nid
                        ));
                    }
                }
                // Hide all lines (they'll be revealed during animation)
                let mut r = ren.borrow_mut();
                for nid in &node_ids {
                    let _ = r.set_visible(NodeId(*nid), false);
                }
            }

            // ── Parse effect-specific kwargs ───────────────────────────
            let fill_rgba = match kwargs.get("fill") {
                Some(Value::List(vals)) if vals.len() == 4 => {
                    let mut rgba = [0.0f32; 4];
                    for (i, v) in vals.iter().enumerate() {
                        if let Value::Number(n) = v {
                            rgba[i] = *n as f32;
                        }
                    }
                    Some(rgba)
                }
                _ => None,
            };

            let to_x = match kwargs.get("x") {
                Some(Value::Number(n)) => Some(*n as f32),
                _ => None,
            };
            let to_y = match kwargs.get("y") {
                Some(Value::Number(n)) => Some(*n as f32),
                _ => None,
            };

            // ── Push animation entries ─────────────────────────────────
            let mut anims = an.borrow_mut();

            for (i, nid) in node_ids.iter().enumerate() {
                let item_delay = per_item_delay(i);

                let kind: AnimationKind = match effect.as_str() {
                    "FadeIn" => AnimationKind::Fade {
                        from_opacity: 0.0,
                        to_opacity: 1.0,
                    },
                    "FadeOut" => AnimationKind::Fade {
                        from_opacity: 1.0,
                        to_opacity: 0.0,
                    },
                    "Grow" => AnimationKind::Scale {
                        from: None,
                        to: 1.0,
                    },
                    "Shrink" => AnimationKind::Scale {
                        from: None,
                        to: 0.0,
                    },
                    "DrawThenFill" => {
                        let rgba = fill_rgba.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                        AnimationKind::DrawThenFill {
                            from_scale: None,
                            fill_rgba: rgba,
                        }
                    }
                    "MoveTo" => {
                        let x = to_x.unwrap_or(0.0);
                        let y = to_y.unwrap_or(0.0);
                        AnimationKind::MoveTo {
                            from_x: None,
                            from_y: None,
                            to_x: x,
                            to_y: y,
                        }
                    }
                    "LineDraw" => AnimationKind::LineDraw,
                    "Morph" => {
                        let morph_src = morph_source_ids.as_ref().expect("already checked");
                        // Pair target[i] with source[i], or last source if out of range
                        let src_id = if i < morph_src.len() {
                            morph_src[i]
                        } else {
                            *morph_src.last().unwrap()
                        };
                        let all_src_ids = morph_src.clone();
                        let r = ren.borrow_mut();
                        let target_cmds = crate::node_to_path_commands(*nid, &r)
                            .map_err(|e| format!("Morph target #{}: {}", i, e))?;
                        let source_cmds = crate::node_to_path_commands(src_id, &r)
                            .map_err(|e| format!("Morph source #{}: {}", i, e))?;
                        let num_samples = ((source_cmds.len() + target_cmds.len()) * 10).max(100);
                        let source_points = sample_path_uniformly(&source_cmds, num_samples);
                        let target_points = sample_path_uniformly(&target_cmds, num_samples);
                        AnimationKind::Morph {
                            source_points,
                            target_points,
                            source_ids: all_src_ids,
                            restore_cmds: target_cmds,
                        }
                    }
                    other => {
                        return Err(format!(
                            "Animate: unknown effect '{other}'. \
                             Available effects: FadeIn, FadeOut, Grow, Shrink, \
                             DrawThenFill, MoveTo, LineDraw, Morph"
                        ));
                    }
                };

                let was_hidden = effect == "LineDraw" || effect == "Morph";

                let entry_duration = per_line_override.unwrap_or(duration);
                anims.push(AnimationEntry {
                    node_id: *nid,
                    duration: entry_duration,
                    delay: item_delay,
                    start_time: None,
                    easing,
                    kind,
                    was_hidden,
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

            // Read global delay from kwargs (all chars shifted by this amount)
            let global_delay = kwarg_num(&kwargs, "delay", 0.0);

            // Build kwargs for each character (skip font/size/position, pass through rest)
            let mut char_kwargs = kwargs.clone();
            char_kwargs.remove("font");
            char_kwargs.remove("size");
            char_kwargs.remove("duration");
            char_kwargs.remove("per_char");
            char_kwargs.remove("delay");
            char_kwargs.remove("segment_subdivisions");
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
                let delay = global_delay + i as f64 * per_char_dur;
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
/// Renders text character-by-character with a path-by-path draw-then-fill reveal.
/// For each character, the outline path is broken into individual segments
/// (LineTo, CubicTo, Close). Phase 1 draws these segments one by one along the
/// outline (like a pen tracing the character). Phase 2 fades the fill in.
/// Similar to Manim's `DrawBorderThenFill`.
///
/// Returns a `List` of NodeIds (one per character — the fill entity ID).
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

            // Read global delay from kwargs (all chars shifted by this amount)
            let global_delay = kwarg_num(&kwargs, "delay", 0.0);

            // Build kwargs for each character (skip font/size/position, pass through rest)
            let mut char_kwargs = kwargs.clone();
            char_kwargs.remove("font");
            char_kwargs.remove("size");
            char_kwargs.remove("duration");
            char_kwargs.remove("per_char");
            char_kwargs.remove("delay");
            char_kwargs.remove("segment_subdivisions");
            char_kwargs.remove("easing");
            char_kwargs.remove("x");
            char_kwargs.remove("y");

            // Extract fill color (values are 0–1 range)
            let fill_color: [f32; 4] = match kwargs.get("fill") {
                Some(Value::List(vals)) if vals.len() >= 3 => [
                    num(Some(&vals[0])) as f32,
                    num(Some(&vals[1])) as f32,
                    num(Some(&vals[2])) as f32,
                    if vals.len() >= 4 { num(Some(&vals[3])) as f32 } else { 1.0 },
                ],
                _ => [1.0, 1.0, 1.0, 1.0],
            };
            let global_opacity = kwarg_num(&kwargs, "opacity", 1.0) as f32;
            let fill_rgba = [fill_color[0], fill_color[1], fill_color[2], fill_color[3] * global_opacity];

            let mut r = ren.borrow_mut();
            let mut node_ids: Vec<Value> = Vec::with_capacity(glyphs.len());

            for (i, glyph) in glyphs.iter().enumerate() {
                // Break the glyph outline into individual drawable segments,
                // then subdivide for smoother progressive drawing.
                let segment_subdivisions = kwarg_num(&kwargs, "segment_subdivisions", 1.0) as u32;
                let raw_segments = extract_path_segments(&glyph.commands);
                let segments = subdivide_segments(&raw_segments, segment_subdivisions);

                // ── Fill entity ──
                // Spawn the full path with fill only (no stroke), hidden initially
                let mut fill_ent_kwargs = char_kwargs.clone();
                fill_ent_kwargs.remove("stroke");
                fill_ent_kwargs.remove("stroke_width");
                fill_ent_kwargs.remove("cap");
                let sk_fill = shapes::parse_shape_kwargs(&fill_ent_kwargs);
                let fill_id = shapes::spawn_2d_shape_with_kwargs(
                    &mut r, x, y, glyph.commands.clone(), &sk_fill,
                );
                let _ = r.set_visible(NodeId(fill_id as u64), false);

                // ── Stroke segment entities ──
                // Each segment is a stroke-only sub-path, hidden initially
                let mut seg_kwargs = char_kwargs.clone();
                seg_kwargs.insert("fill".to_string(), Value::List(vec![])); // explicit no-fill
                let sk_seg = shapes::parse_shape_kwargs(&seg_kwargs);
                let mut seg_ids: Vec<u64> = Vec::with_capacity(segments.len());
                for seg_cmds in &segments {
                    let seg_id = shapes::spawn_2d_shape_with_kwargs(
                        &mut r, x, y, seg_cmds.clone(), &sk_seg,
                    );
                    let _ = r.set_visible(NodeId(seg_id as u64), false);
                    seg_ids.push(seg_id as u64);
                }

                // Register a PathDrawThenFill animation for this character
                let delay = global_delay + i as f64 * per_char_dur;
                an.borrow_mut().push(AnimationEntry {
                    node_id: fill_id as u64, // primary node for reference
                    duration: per_char_dur,
                    delay,
                    start_time: None,
                    easing: easing_name,
                    kind: AnimationKind::PathDrawThenFill {
                        segment_ids: seg_ids,
                        fill_rgba,
                        fill_entity_id: fill_id as u64,
                        initialized: false,
                    },
                    was_hidden: false, // all visibility managed by PathDrawThenFill dispatch
                });

                node_ids.push(Value::NodeId(fill_id));
            }

            Ok(Value::List(node_ids))
        })),
    );
}

// ─── Morph ────────────────────────────────────────────────────────────────────

/// Morph(target, source, duration: 1.0, delay: 0.0, easing: "ease_out_cubic")
///
/// Animates the path of `target` so it morphs from `source`'s shape into
/// `target`'s own shape. Both nodes must have `PathData` (created by any shape
/// function: `Circle`, `Rect`, `Polygon`, `Tex()`, etc.).
///
/// The morph works by extracting all vertices/control points from both paths,
/// padding the shorter sequence to match, and linearly interpolating each point
/// every frame. The reconstructed path uses MoveTo/LineTo/Close so curved
/// segments become polylines during the morph — a common approach in Manim-style
/// animations.
///
/// # Arguments
/// * `target` — NodeId of the node to animate (morphs into its own final shape)
/// * `source` — NodeId of the node whose shape is the starting state
///
/// # Example
/// ```scalar
/// a = Circle(100, 100, 50)
/// b = Rect(100, 100, 120, 80)
/// Morph(b, a, duration: 2.0)    // rect morphs from circle
/// ```
fn register_morph(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    animations: Rc<RefCell<Vec<AnimationEntry>>>,
) {
    let ren = renderer.clone();
    let an = animations.clone();
    env.define(
        "Morph".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 2 {
                return Err("Morph(target, source, ...): requires at least 2 arguments (target and source node IDs)".to_string());
            }

            let ap = parse_anim_params(&kwargs);

            let r = ren.borrow_mut();

            // Resolve all target node IDs from first argument (NodeId, Number, or [NodeId])
            let target_ids: Vec<u64> = crate::value_to_ids(&args[0])?;
            // Resolve all source node IDs from second argument
            let source_ids: Vec<u64> = crate::value_to_ids(&args[1])?;

            if target_ids.is_empty() {
                return Err("Morph: target has no valid node IDs".to_string());
            }
            if source_ids.is_empty() {
                return Err("Morph: source has no valid node IDs".to_string());
            }

            // Create one Morph entry per pair, aligning lengths by repeating last source
            let max_len = target_ids.len().max(source_ids.len());
            let get_source = |i: usize| -> u64 {
                if i < source_ids.len() { source_ids[i] } else { *source_ids.last().unwrap() }
            };

            let mut anims = an.borrow_mut();
            for i in 0..target_ids.len() {
                let tgt_id = target_ids[i];
                let src_id = get_source(i);

                // Extract PathCommands for target glyph
                let target_cmds = match crate::node_to_path_commands(tgt_id, &r) {
                    Ok(cmds) => cmds,
                    Err(e) => return Err(format!("Morph target #{}: {}", i, e)),
                };
                // Extract PathCommands for source glyph
                let source_cmds = match crate::node_to_path_commands(src_id, &r) {
                    Ok(cmds) => cmds,
                    Err(e) => return Err(format!("Morph source #{}: {}", i, e)),
                };

                let num_samples = ((source_cmds.len() + target_cmds.len()) * 10).max(100);
                let source_points = sample_path_uniformly(&source_cmds, num_samples);
                let target_points = sample_path_uniformly(&target_cmds, num_samples);

                anims.push(AnimationEntry {
                    node_id: tgt_id,
                    duration: ap.duration,
                    delay: ap.delay,
                    start_time: None,
                    easing: ap.easing,
                    kind: AnimationKind::Morph {
                        source_points,
                        target_points,
                        source_ids: source_ids.clone(),
                        restore_cmds: target_cmds.clone(),
                    },
                    was_hidden: true,  // show on first morph frame (user hides target beforehand)
                });
            }

            Ok(Value::Number(target_ids.len() as f64))
        })),
    );
}
