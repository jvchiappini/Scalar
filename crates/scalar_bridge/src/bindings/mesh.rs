use std::rc::Rc;
use std::cell::RefCell;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, Transform, glam::Vec3};

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    // spawn_mesh(name, x, y, z) -> NodeId
    let r = renderer.clone();
    env.define("spawn_mesh".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if args.len() != 4 {
             return Err("spawn_mesh expects 4 arguments: (name, x, y, z)".to_string());
        }
        let name = match &args[0] { Value::String(s) => s.clone(), _ => return Err("name must be a string".into()) };
        let x = match args[1] { Value::Number(n) => n as f32, _ => return Err("x must be a number".into()) };
        let y = match args[2] { Value::Number(n) => n as f32, _ => return Err("y must be a number".into()) };
        let z = match args[3] { Value::Number(n) => n as f32, _ => return Err("z must be a number".into()) };
        
        let mut renderer_mut = r.borrow_mut();
        let id = renderer_mut.spawn_mesh(&name, Transform::from_position(Vec3::new(x, y, z)));
        Ok(Value::NodeId(id.0 as u32))
    })));

    // Cube(x, y, z) alias
    let r = renderer.clone();
    env.define("Cube".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let x = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let y = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let z = match args.get(2) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let id = r.borrow_mut().spawn_mesh("cube", Transform::from_position(Vec3::new(x, y, z)));
        Ok(Value::NodeId(id.0 as u32))
    })));

    // Sphere(x, y, z, radius) alias
    let r = renderer.clone();
    env.define("Sphere".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        let x = match args.get(0) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let y = match args.get(1) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let z = match args.get(2) { Some(Value::Number(n)) => *n as f32, _ => 0.0 };
        let rad = match args.get(3) { Some(Value::Number(n)) => *n as f32, _ => 1.0 };
        
        let mut ren = r.borrow_mut();
        let id = ren.spawn_mesh("sphere", Transform::from_position(Vec3::new(x, y, z)).with_scale(Vec3::splat(rad)));
        Ok(Value::NodeId(id.0 as u32))
    })));
}
