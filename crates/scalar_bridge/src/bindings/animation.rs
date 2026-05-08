use std::rc::Rc;
use std::cell::RefCell;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, Animator, AnimJob, EasingType, Transform, MaterialComponent};

/// Registers the animation sequencer bindings.
pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, playhead: Rc<RefCell<f64>>) {
    // wait(seconds)
    // Advances the virtual playhead.
    let ph = playhead.clone();
    env.define("wait".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if args.len() != 1 {
            return Err("wait() expects 1 argument: seconds".into());
        }
        if let Value::Number(s) = args[0] {
            *ph.borrow_mut() += s;
                Ok(Value::Number(0.0))
        } else {
            Err("wait(): argument must be a number".into())
        }
    })));

    // animate(node_id, property_str, end_val, duration, easing_type)
    // Schedules an animation job starting at the current playhead position.
    let ph = playhead.clone();
    let r = renderer.clone();
    env.define("animate".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if args.len() < 5 {
            return Err("animate() expects 5 arguments: (node_id, property, end_val, duration, easing)".into());
        }

        let node_id = match args[0] { 
            Value::NodeId(id) => ferrous_engine::NodeId(id as u64), 
            _ => return Err("animate(): first arg must be a NodeId".into()) 
        };
        let property = match &args[1] { 
            Value::String(s) => s.clone(), 
            _ => return Err("animate(): second arg must be a String property name".into()) 
        };
        let end_val = match args[2] { 
            Value::Number(n) => n as f32, 
            _ => return Err("animate(): third arg must be a Number end_val".into()) 
        };
        let duration = match args[3] { 
            Value::Number(n) => n, 
            _ => return Err("animate(): fourth arg must be a Number duration".into()) 
        };
        let easing_str = match &args[4] { 
            Value::String(s) => s.as_str(), 
            _ => return Err("animate(): fifth arg must be a String easing_type".into()) 
        };

        let easing = match easing_str {
            "linear" | "Linear" => EasingType::Linear,
            "easeInOutCubic" | "EaseInOut" => EasingType::EaseInOutCubic,
            "elasticOut" | "ElasticOut" => EasingType::ElasticOut,
            _ => EasingType::Linear,
        };

        let start_time = *ph.borrow();

        let mut ren = r.borrow_mut();
        let entity = *ren.node_map.get(&node_id).ok_or_else(|| format!("Node {} not found", node_id.0))?;
        
        let world = &mut ren.world.ecs;
        
        // Lookup start_val from current ECS state
        let start_val = match property.as_str() {
            "x" => world.get::<Transform>(entity).map(|t| t.position.x).unwrap_or(0.0),
            "y" => world.get::<Transform>(entity).map(|t| t.position.y).unwrap_or(0.0),
            "z" => world.get::<Transform>(entity).map(|t| t.position.z).unwrap_or(0.0),
            "scale" => world.get::<Transform>(entity).map(|t| t.scale.x).unwrap_or(1.0),
            "opacity" => world.get::<MaterialComponent>(entity).map(|m| m.descriptor.opacity).unwrap_or(1.0),
            "r" => world.get::<MaterialComponent>(entity).map(|m| m.descriptor.base_color[0]).unwrap_or(1.0),
            "g" => world.get::<MaterialComponent>(entity).map(|m| m.descriptor.base_color[1]).unwrap_or(1.0),
            "b" => world.get::<MaterialComponent>(entity).map(|m| m.descriptor.base_color[2]).unwrap_or(1.0),
            "a" => world.get::<MaterialComponent>(entity).map(|m| m.descriptor.base_color[3]).unwrap_or(1.0),
            _ => 0.0,
        };

        let job = AnimJob {
            property,
            start_val,
            end_val,
            start_time,
            duration,
            easing,
            path_start: None,
            path_target: None,
        };

        if let Some(mut animator) = world.get_mut::<Animator>(entity) {
            animator.jobs.push(job.clone());
        } else {
            world.insert(entity, Animator { jobs: vec![job.clone()] });
        }

        // TELEPORTATION FIX (Declaration-Execution sync)
        // Immediately update the ECS value at t=0 so the NEXT animate() in the script
        // reads this as its start_val.
        match job.property.as_str() {
            "x" => if let Some(mut tr) = world.get_mut::<Transform>(entity) { tr.position.x = end_val; },
            "y" => if let Some(mut tr) = world.get_mut::<Transform>(entity) { tr.position.y = end_val; },
            "z" => if let Some(mut tr) = world.get_mut::<Transform>(entity) { tr.position.z = end_val; },
            "scale" => if let Some(mut tr) = world.get_mut::<Transform>(entity) { tr.scale = ferrous_engine::glam::Vec3::splat(end_val); },
            "opacity" => if let Some(mut mc) = world.get_mut::<MaterialComponent>(entity) { 
                mc.descriptor.opacity = end_val;
                mc.descriptor.base_color[3] = end_val;
            },
            "r" => if let Some(mut mc) = world.get_mut::<MaterialComponent>(entity) { mc.descriptor.base_color[0] = end_val; },
            "g" => if let Some(mut mc) = world.get_mut::<MaterialComponent>(entity) { mc.descriptor.base_color[1] = end_val; },
            "b" => if let Some(mut mc) = world.get_mut::<MaterialComponent>(entity) { mc.descriptor.base_color[2] = end_val; },
            "a" => if let Some(mut mc) = world.get_mut::<MaterialComponent>(entity) { mc.descriptor.base_color[3] = end_val; },
            _ => {}
        }

            Ok(Value::Number(0.0))
    })));
}
