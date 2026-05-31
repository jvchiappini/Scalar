use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::{Renderer, NodeId};
use crate::{LineData, AnimatingLine};

pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    line_data: Rc<RefCell<HashMap<u64, LineData>>>,
    animations: Rc<RefCell<Vec<AnimatingLine>>>,
) {
    register_set_line_progress(env, renderer.clone(), line_data.clone());
    register_set_line_cap(env, renderer.clone());
    register_animate(env, renderer.clone(), line_data.clone(), animations.clone());
}

/// SetLineCap(node_id, cap)
///
/// Cambia el estilo de punta de una línea.
/// `cap` puede ser: "flat" (0), "round" (1), "square" (2).
fn register_set_line_cap(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
) {
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
    animations: Rc<RefCell<Vec<AnimatingLine>>>,
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

            // Hide all lines before animation begins (they'll be revealed as they draw)
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
                anims.push(AnimatingLine {
                    node_id: *nid,
                    duration: per_line_dur,
                    delay,
                    start_time: None,
                    easing: easing_name,
                    was_hidden: true,
                });
            }

            Ok(Value::Number(node_ids.len() as f64))
        })),
    );
}
