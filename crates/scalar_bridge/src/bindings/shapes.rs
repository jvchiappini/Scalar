use std::rc::Rc;
use std::cell::RefCell;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, NodeId, glam::Vec3, Transform};

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    // Circle(radius)
    let r = renderer.clone();
    env.define("Circle".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let radius = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_circle(0.0, 0.0, 0.0, radius);
        Ok(Value::NodeId(id.0 as u32))
    })));

    // Rect(x, y, z, w, h)
    let r = renderer.clone();
    env.define("Rect".to_string(), Value::NativeFunction(Rc::new(move |args| {
        let x = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let y = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let z = match args.get(2) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let w = match args.get(3) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        let h = match args.get(4) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        
        let mut ren = r.borrow_mut();
        let id = ren.spawn_2d_rect(x, y, z, w, h);
        Ok(Value::NodeId(id.0 as u32))
    })));

    // set_color(node, r, g, b, a)
    let r = renderer.clone();
    env.define("set_color".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::NodeId(id), Value::Number(rv), Value::Number(gv), Value::Number(bv), Value::Number(av)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let node = NodeId(*id as u64);
            let _ = ren.update_color(node, [*rv as f32, *gv as f32, *bv as f32, *av as f32]);
            Ok(Value::Number(0.0))
        } else {
            Err("set_color(node, r, g, b, a) expected".into())
        }
    })));

    // set_position(node, x, y, z)
    let r = renderer.clone();
    env.define("set_position".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::NodeId(id), Value::Number(x), Value::Number(y), Value::Number(z)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let node = NodeId(*id as u64);
            let _ = ren.update_transform(node, Transform::from_position(Vec3::new(*x as f32, *y as f32, *z as f32)));
            Ok(Value::Number(0.0))
        } else {
            Err("set_position(node, x, y, z) expected".into())
        }
    })));

    // set_shadow_caster(node, state)
    let r = renderer.clone();
    env.define("set_shadow_caster".to_string(), Value::NativeFunction(Rc::new(move |args| {
        if let [Value::NodeId(id), Value::Number(state)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let node = NodeId(*id as u64);
            let _ = ren.set_shadow_caster(node, *state > 0.5);
            Ok(Value::Number(0.0))
        } else {
            Err("set_shadow_caster(node, state) expected".into())
        }
    })));

    // clear_scene()
    let r = renderer.clone();
    env.define("clear_scene".to_string(), Value::NativeFunction(Rc::new(move |_| {
        r.borrow_mut().clear();
        Ok(Value::Number(0.0))
    })));
}
