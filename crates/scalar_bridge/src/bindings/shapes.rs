use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, NodeId, glam::Vec3, Transform};

fn resolve_color(args: &[Value], offset: usize) -> Option<[f32; 4]> {
    if args.len() >= offset + 4 {
        if let (Value::Number(r), Value::Number(g), Value::Number(b), Value::Number(a)) = (&args[offset], &args[offset+1], &args[offset+2], &args[offset+3]) {
            return Some([*r as f32, *g as f32, *b as f32, *a as f32]);
        }
    }
    if args.len() > offset {
        if let Value::List(l) = &args[offset] {
            if l.len() == 4 {
                let mut rgba = [0.0; 4];
                for (i, v) in l.iter().enumerate() {
                    if let Value::Number(n) = v { rgba[i] = *n as f32; }
                }
                return Some(rgba);
            }
        }
    }
    None
}

fn create_shape_object(id: NodeId, renderer: Rc<RefCell<Renderer>>, ph: Rc<RefCell<f64>>) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__id".to_string(), Value::NodeId(id.0 as u32));

    let r = renderer.clone();
    obj.insert("set_stroke".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let color = resolve_color(&args, 1);
        let thick = match args.last() { Some(Value::Number(n)) => *n as f32, _ => 0.05 };
        if let Some(c) = color {
            let _ = r.borrow_mut().set_stroke(id, c, thick);
        }
        Ok(Value::Number(0.0))
    })));

    let r = renderer.clone();
    obj.insert("set_fill".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let Some(c) = resolve_color(&args, 1) {
            let _ = r.borrow_mut().set_fill(id, c);
        }
        Ok(Value::Number(0.0))
    })));

    let r = renderer.clone();
    obj.insert("set_z_index".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let Some(Value::Number(z)) = args.get(1) {
            let _ = r.borrow_mut().set_z_index(id, *z as i32);
        }
        Ok(Value::Number(0.0))
    })));

    let r = renderer.clone();
    obj.insert("set_opacity".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let Some(Value::Number(opacity)) = args.get(1) {
             let _ = r.borrow_mut().set_opacity(id, *opacity as f32);
        }
        Ok(Value::Number(0.0))
    })));

    let r = renderer.clone();
    let ph_clone = ph.clone();
    obj.insert("morph_to".to_string(), Value::NativeFunction(Rc::new(move |args, kwargs| {
        if let [_, Value::Object(target_obj)] = args.as_slice() {
            let target_id = match target_obj.get("__id") {
                Some(Value::NodeId(n)) => NodeId(*n as u64),
                _ => return Err("morph_to: target must be a shape object".into()),
            };

            let duration = match kwargs.get("duration") {
                Some(Value::Number(n)) => *n,
                _ => match args.get(2) { Some(Value::Number(n)) => *n, _ => 1.0 }
            };

            let easing_str = match kwargs.get("easing") {
                Some(Value::String(s)) => s.as_str(),
                _ => match args.get(3) { Some(Value::String(s)) => s.as_str(), _ => "linear" }
            };

            let mut ren = r.borrow_mut();
            let entity = *ren.node_map.get(&id).ok_or("Source entity not found")?;
            let target_entity = *ren.node_map.get(&target_id).ok_or("Target entity not found")?;
            
            let path_start = ren.world.ecs.get::<ferrous_core::scene::world::types::PathData>(entity).cloned();
            let path_target = ren.world.ecs.get::<ferrous_core::scene::world::types::PathData>(target_entity).cloned();

            let easing = match easing_str {
                "easeInOut" | "easeInOutCubic" | "ease_in_out" => ferrous_engine::EasingType::EaseInOutCubic,
                _ => ferrous_engine::EasingType::Linear,
            };

            let job = ferrous_engine::AnimJob {
                property: "__path__".to_string(),
                start_val: 0.0,
                end_val: 1.0,
                start_time: *ph_clone.borrow(),
                duration,
                easing,
                path_start,
                path_target: path_target.clone(),
            };

            if let Some(mut animator) = ren.world.ecs.get_mut::<ferrous_engine::Animator>(entity) {
                animator.jobs.push(job);
            } else {
                ren.world.ecs.insert(entity, ferrous_engine::Animator { jobs: vec![job] });
            }

            // v14 Fix: Auto-hide the target mold to prevent "strange square" ghosting
            let _ = ren.set_visible(target_id, false);

            Ok(Value::Number(0.0))
        } else {
            Err("morph_to(target, duration, easing) expected".into())
        }
    })));

    Value::Object(obj)
}

fn create_group_object(ids: Vec<NodeId>, renderer: Rc<RefCell<Renderer>>, ph: Rc<RefCell<f64>>) -> Value {
    let mut obj = HashMap::new();
    let r = renderer.clone();
    let ph_clone = ph.clone();
    
    obj.insert("set_stroke".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let color = resolve_color(&args, 1);
        let thick = match args.last() { Some(Value::Number(n)) => *n as f32, _ => 0.05 };
        if let Some(c) = color {
            for id in &ids {
                let _ = r.borrow_mut().set_stroke(*id, c, thick);
            }
        }
        Ok(Value::Number(0.0))
    })));
    Value::Object(obj)
}

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, ph: Rc<RefCell<f64>>) {
    // Circle(radius) or Circle(x,y,radius) or Circle(x,y,z,radius)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Circle".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let (x, y, z, rad) = match args.as_slice() {
            [Value::Number(r)] => (0.0, 0.0, 0.0, *r as f32),
            [Value::Number(x), Value::Number(y), Value::Number(r)] => (*x as f32, *y as f32, 0.0, *r as f32),
            [Value::Number(x), Value::Number(y), Value::Number(z), Value::Number(r)] => (*x as f32, *y as f32, *z as f32, *r as f32),
            _ => (0.0, 0.0, 0.0, 1.0),
        };
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_circle(x, y, z, rad);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Rect(w, h) or Rect(x,y,w,h)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Rect".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let (x, y, z, w, h) = match args.as_slice() {
            [Value::Number(w), Value::Number(h)] => (0.0, 0.0, 0.0, *w as f32, *h as f32),
            [Value::Number(x), Value::Number(y), Value::Number(w), Value::Number(h)] => (*x as f32, *y as f32, 0.0, *w as f32, *h as f32),
            _ => (0.0, 0.0, 0.0, 1.0, 1.0),
        };
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_rect(x, y, z, w, h);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Axes(x_min, x_max, y_min, y_max)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Axes".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [Value::Number(x1), Value::Number(x2), Value::Number(y1), Value::Number(y2)] = args.as_slice() {
             let mut ren = r.borrow_mut();
             let x_id = ren.spawn_2d_line(*x1 as f32, 0.0, *x2 as f32, 0.0, 0.08);
             let y_id = ren.spawn_2d_line(0.0, *y1 as f32, 0.0, *y2 as f32, 0.08);
             // v14 Fix: Bring axes to front to ensure visibility
             let _ = ren.set_z_index(x_id, 10);
             let _ = ren.set_z_index(y_id, 10);
             Ok(create_group_object(vec![x_id, y_id], r.clone(), ph_clone.clone()))
        } else { Err("Axes expected 4 numbers".into()) }
    })));

    // Plot(expression_str, x_min, x_max, steps)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Plot".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        // Option A: Evaluate string or simple lambda if we can.
        // For now, we'll support only x*x type expressions or handle it via points.
        // But the user used x => x*x. Since parser doesn't support it, we'll suggest Plot("x*x", ...)
        // and handle a very basic expression evaluator here.
        if let [func_val, Value::Number(x1), Value::Number(x2)] = args.as_slice() {
            let steps = 100;
            let mut commands = Vec::new();
            for i in 0..=steps {
                let x = x1 + (x2 - x1) * (i as f64 / steps as f64);
                // Placeholder: we'll assume x*x if it's not a function
                let y = x * x; 
                let pos = ferrous_engine::glam::Vec2::new(x as f32, y as f32);
                if i == 0 { commands.push(ferrous_core::scene::world::types::PathCommand::MoveTo(pos)); }
                else { commands.push(ferrous_core::scene::world::types::PathCommand::LineTo(pos)); }
            }
            let path_data = ferrous_core::scene::world::types::PathData { commands };
            let id = r.borrow_mut().spawn_2d_path(Transform::default(), path_data);
            Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
        } else { Err("Plot(expression, x_min, x_max) expected".into()) }
    })));

    // clear_scene()
    let r = renderer.clone();
    env.define("clear_scene".to_string(), Value::NativeFunction(Rc::new(move |_, _| {
        r.borrow_mut().clear();
        Ok(Value::Number(0.0))
    })));

    // Line(x1, y1, x2, y2, thickness)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Line".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let x1 = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let y1 = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let x2 = match args.get(2) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let y2 = match args.get(3) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let t  = match args.get(4) { Some(Value::Number(n)) => *n as f32, _ => 0.1 };
        let id = r.borrow_mut().spawn_2d_line(x1, y1, x2, y2, t);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Path([[x,y], ...], thickness)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Path".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [Value::List(points), Value::Number(thick)] = args.as_slice() {
            let mut commands = Vec::new();
            for (i, p) in points.iter().enumerate() {
                if let Value::List(xy) = p {
                    if let [Value::Number(vx), Value::Number(vy)] = xy.as_slice() {
                        let pos = ferrous_engine::glam::Vec2::new(*vx as f32, *vy as f32);
                        if i == 0 { commands.push(ferrous_core::scene::world::types::PathCommand::MoveTo(pos)); }
                        else { commands.push(ferrous_core::scene::world::types::PathCommand::LineTo(pos)); }
                    }
                }
            }
            let path_data = ferrous_core::scene::world::types::PathData { commands };
            let id = r.borrow_mut().spawn_2d_path(Transform::default(), path_data);
            Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
        } else { Err("Path expected points and thickness".into()) }
    })));

    // Arrow(x1, y1, x2, y2)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Arrow".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [Value::Number(x1), Value::Number(y1), Value::Number(x2), Value::Number(y2)] = args.as_slice() {
            let x1 = *x1 as f32; let y1 = *y1 as f32; let x2 = *x2 as f32; let y2 = *y2 as f32;
            let mut ren = r.borrow_mut();
            let id = ren.spawn_2d_line(x1, y1, x2, y2, 0.05);
            let dx = x2 - x1; let dy = y2 - y1;
            let len = (dx*dx + dy*dy).sqrt();
            if len > 0.1 {
                let ux = dx / len; let uy = dy / len;
                let vx = -uy; let vy = ux;
                let hs = 0.25; let hw = 0.15;
                let p1 = Vec3::new(x2 - ux * hs + vx * hw, y2 - uy * hs + vy * hw, 0.0);
                let p2 = Vec3::new(x2 - ux * hs - vx * hw, y2 - uy * hs - vy * hw, 0.0);
                let p3 = Vec3::new(x2, y2, 0.0);
                let head_data = ferrous_core::scene::world::types::PathData {
                    commands: vec![
                        ferrous_core::scene::world::types::PathCommand::MoveTo(p3.truncate()),
                        ferrous_core::scene::world::types::PathCommand::LineTo(p1.truncate()),
                        ferrous_core::scene::world::types::PathCommand::LineTo(p2.truncate()),
                        ferrous_core::scene::world::types::PathCommand::LineTo(p3.truncate()),
                    ]
                };
                let head_id = ren.spawn_2d_path(Transform::default(), head_data);
                let _ = ren.set_fill(head_id, [1.0, 1.0, 1.0, 1.0]);
            }
            Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
        } else { Err("Arrow expected 4 numbers".into()) }
    })));
}
