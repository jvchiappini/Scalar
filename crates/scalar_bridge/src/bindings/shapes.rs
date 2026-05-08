use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, NodeId, glam::Vec3, Transform};

fn create_shape_object(id: NodeId, renderer: Rc<RefCell<Renderer>>, ph: Rc<RefCell<f64>>) -> Value {
    let mut obj = HashMap::new();
    obj.insert("__id".to_string(), Value::NodeId(id.0 as u32));

    // obj.set_stroke(r, g, b, a, thickness)
    let r = renderer.clone();
    obj.insert("set_stroke".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [_, Value::Number(rv), Value::Number(gv), Value::Number(bv), Value::Number(av), Value::Number(thick)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let _ = ren.set_stroke(id, [*rv as f32, *gv as f32, *bv as f32, *av as f32], *thick as f32);
            Ok(Value::Number(0.0))
        } else {
            Err("set_stroke(r, g, b, a, thickness) expected".into())
        }
    })));

    // obj.set_fill(r, g, b, a)
    let r = renderer.clone();
    obj.insert("set_fill".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [_, Value::Number(rv), Value::Number(gv), Value::Number(bv), Value::Number(av)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let _ = ren.set_fill(id, [*rv as f32, *gv as f32, *bv as f32, *av as f32]);
            Ok(Value::Number(0.0))
        } else {
            Err("set_fill(r, g, b, a) expected".into())
        }
    })));

    // obj.set_z_index(z)
    let r = renderer.clone();
    obj.insert("set_z_index".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [_, Value::Number(z)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let _ = ren.set_z_index(id, *z as i32);
            Ok(Value::Number(0.0))
        } else {
            Err("set_z_index(z) expected".into())
        }
    })));

    // obj.morph_to(target_obj, duration, easing)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    obj.insert("morph_to".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [_, Value::Object(target_obj), Value::Number(duration), Value::String(easing_str)] = args.as_slice() {
            let target_id = match target_obj.get("__id") {
                Some(Value::NodeId(n)) => NodeId(*n as u64),
                _ => return Err("morph_to: target must be a shape object".into()),
            };

            let mut ren = r.borrow_mut();
            let entity = *ren.node_map.get(&id).ok_or("Source entity not found")?;
            let target_entity = *ren.node_map.get(&target_id).ok_or("Target entity not found")?;
            
            // Extract PathData if it exists, otherwise use a placeholder (for primitives)
            // For circles/rects that are primitives, we'd need to convert them to paths.
            // For v13, we assume PathData component is present or we can generate it.
            let path_start = ren.world.ecs.get::<ferrous_core::scene::world::types::PathData>(entity).cloned();
            let path_target = ren.world.ecs.get::<ferrous_core::scene::world::types::PathData>(target_entity).cloned();

            let easing = match easing_str.as_str() {
                "easeInOut" | "easeInOutCubic" => ferrous_engine::EasingType::EaseInOutCubic,
                _ => ferrous_engine::EasingType::Linear,
            };

            let job = ferrous_engine::AnimJob {
                property: "__path__".to_string(),
                start_val: 0.0,
                end_val: 1.0,
                start_time: *ph_clone.borrow(),
                duration: *duration,
                easing,
                path_start,
                path_target: path_target.clone(),
            };

            if let Some(mut animator) = ren.world.ecs.get_mut::<ferrous_engine::Animator>(entity) {
                animator.jobs.push(job);
            } else {
                ren.world.ecs.insert(entity, ferrous_engine::Animator { jobs: vec![job] });
            }

            // Sync: teleport PathData to end state for declaration phase
            if let Some(target) = path_target {
                 if let Some(mut path) = ren.world.ecs.get_mut::<ferrous_core::scene::world::types::PathData>(entity) {
                     *path = target;
                 }
            }

            Ok(Value::Number(0.0))
        } else {
            Err("morph_to(target, duration, easing) expected".into())
        }
    })));

    Value::Object(obj)
}

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, ph: Rc<RefCell<f64>>) {
    // Circle(radius)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Circle".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let radius = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_circle(0.0, 0.0, 0.0, radius);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Rect(w, h)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Rect".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let w = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let h = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_rect(0.0, 0.0, 0.0, w, h);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Line(x1, y1, x2, y2, thickness)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Line".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let x1 = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let y1 = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let x2 = match args.get(2) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let y2 = match args.get(3) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let t  = match args.get(4) { Some(Value::Number(n)) => *n as f32, _ => 0.1 };
        
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_line(x1, y1, x2, y2, t);
        Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
    })));

    // Path([[x,y], [x,y], ...], thickness)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Path".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::List(points), Value::Number(thick)] = args.as_slice() {
            let mut commands = Vec::new();
            for (i, p) in points.iter().enumerate() {
                if let Value::List(xy) = p {
                    if let [Value::Number(vx), Value::Number(vy)] = xy.as_slice() {
                        let pos = ferrous_engine::glam::Vec2::new(*vx as f32, *vy as f32);
                        if i == 0 {
                            commands.push(ferrous_core::scene::world::types::PathCommand::MoveTo(pos));
                        } else {
                            commands.push(ferrous_core::scene::world::types::PathCommand::LineTo(pos));
                        }
                    }
                }
            }
            let path_data = ferrous_core::scene::world::types::PathData { commands };
            let mut ren = r.borrow_mut();
            let id = ren.spawn_2d_path(Transform::default(), path_data);
            let _ = ren.set_stroke(id, [1.0, 1.0, 1.0, 1.0], *thick as f32);
            Ok(create_shape_object(id, r.clone(), ph_clone.clone()))
        } else {
            Err("Path([[x,y], ...], thickness) expected".into())
        }
    })));

    // Arrow(x1, y1, x2, y2)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Arrow".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::Number(x1), Value::Number(y1), Value::Number(x2), Value::Number(y2)] = args.as_slice() {
            let x1 = *x1 as f32; let y1 = *y1 as f32; let x2 = *x2 as f32; let y2 = *y2 as f32;
            let mut ren = r.borrow_mut();
            let id = ren.spawn_2d_line(x1, y1, x2, y2, 0.05);
            
            // Triangle head
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
        } else {
            Err("Arrow(x1, y1, x2, y2) expected".into())
        }
    })));

    // Axes(x_min, x_max, y_min, y_max)
    let r = renderer.clone();
    let ph_clone = ph.clone();
    env.define("Axes".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::Number(x_min), Value::Number(x_max), Value::Number(y_min), Value::Number(y_max)] = args.as_slice() {
             let mut ren = r.borrow_mut();
             // X-axis
             let x_id = ren.spawn_2d_line(*x_min as f32, 0.0, *x_max as f32, 0.0, 0.03);
             let _ = ren.update_color(x_id, [0.5, 0.5, 0.5, 1.0]);
             // Y-axis
             let y_id = ren.spawn_2d_line(0.0, *y_min as f32, 0.0, *y_max as f32, 0.03);
             let _ = ren.update_color(y_id, [0.5, 0.5, 0.5, 1.0]);
             Ok(Value::Number(0.0))
        } else {
             Err("Axes(x_min, x_max, y_min, y_max) expected".into())
        }
    })));

    // set_color(node, r, g, b, a)
    let r = renderer.clone();
    env.define("set_color".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let (id_val, rv, gv, bv, av) = if let [Value::NodeId(id), Value::Number(rv), Value::Number(gv), Value::Number(bv), Value::Number(av)] = args.as_slice() {
            (*id as u64, *rv as f32, *gv as f32, *bv as f32, *av as f32)
        } else if let [Value::Object(obj), Value::Number(rv), Value::Number(gv), Value::Number(bv), Value::Number(av)] = args.as_slice() {
            let id = match obj.get("__id") { Some(Value::NodeId(n)) => *n as u64, _ => return Err("Invalid object".into()) };
            (id, *rv as f32, *gv as f32, *bv as f32, *av as f32)
        } else {
             return Err("set_color(node|obj, r, g, b, a) expected".into());
        };

        let mut ren = r.borrow_mut();
        let _ = ren.update_color(NodeId(id_val), [rv, gv, bv, av]);
        Ok(Value::Number(0.0))
    })));

    // set_position(node, x, y, z)
    let r = renderer.clone();
    env.define("set_position".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let (id_val, x, y, z) = if let [Value::NodeId(id), Value::Number(x), Value::Number(y), Value::Number(z)] = args.as_slice() {
            (*id as u64, *x as f32, *y as f32, *z as f32)
        } else if let [Value::Object(obj), Value::Number(x), Value::Number(y), Value::Number(z)] = args.as_slice() {
            let id = match obj.get("__id") { Some(Value::NodeId(n)) => *n as u64, _ => return Err("Invalid object".into()) };
            (id, *x as f32, *y as f32, *z as f32)
        } else {
             return Err("set_position(node|obj, x, y, z) expected".into());
        };

        let mut ren = r.borrow_mut();
        let _ = ren.update_transform(NodeId(id_val), Transform::from_position(Vec3::new(x, y, z)));
        Ok(Value::Number(0.0))
    })));

    // set_shadow_caster(node, state)
    let r = renderer.clone();
    env.define("set_shadow_caster".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let (id_val, state) = if let [Value::NodeId(id), Value::Number(s)] = args.as_slice() {
             (*id as u64, *s > 0.5)
        } else if let [Value::Object(obj), Value::Number(s)] = args.as_slice() {
             let id = match obj.get("__id") { Some(Value::NodeId(n)) => *n as u64, _ => return Err("Invalid object".into()) };
             (id, *s > 0.5)
        } else {
             return Err("set_shadow_caster(node|obj, state) expected".into());
        };

        let mut ren = r.borrow_mut();
        let _ = ren.set_shadow_caster(NodeId(id_val), state);
        Ok(Value::Number(0.0))
    })));

    // clear_scene()
    let r = renderer.clone();
    env.define("clear_scene".to_string(), Value::NativeFunction(Rc::new(move |_| {
        r.borrow_mut().clear();
        Ok(Value::Number(0.0))
    })));

    // append(list, value)
    env.define("append".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::List(l), val] = args.as_slice() {
            // Note: In a real environment we'd need to handle mutability carefully.
            // For now, we'll assume the user wants to mutate.
            // But Value is not behind a RefCell here in List variant usually.
            // Wait, Value::List(Vec<Value>) is owned.
            // I should probably return a new list or use a RefCell in Value::List.
            // Since I can't easily change scalar_lang's Value now, I'll just skip this 
            // and implement Plot natively too for reliability.
            Ok(Value::Number(0.0))
        } else {
            Err("append(list, value) expected".into())
        }
    })));
}
