use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::runtime::{Environment, Value};
use ferrous_engine::{Renderer, glam::Vec3};

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let mut camera_obj = HashMap::new();

    // Camera.set_position(x, y, z)
    let r = renderer.clone();
    camera_obj.insert("set_position".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [_, Value::Number(x), Value::Number(y), Value::Number(z)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let cam = ren.camera_mut();
            cam.set_position(Vec3::new(*x as f32, *y as f32, *z as f32));
            Ok(Value::Number(0.0))
        } else {
            Err("Camera.set_position(x, y, z) expected".into())
        }
    })));

    // Camera.look_at(x, y, z)
    let r = renderer.clone();
    camera_obj.insert("look_at".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [_, Value::Number(x), Value::Number(y), Value::Number(z)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let cam = ren.camera_mut();
            let eye = cam.eye;
            cam.look_at(eye, Vec3::new(*x as f32, *y as f32, *z as f32));
            Ok(Value::Number(0.0))
        } else {
            Err("Camera.look_at(x, y, z) expected".into())
        }
    })));

    // Camera.set_rotation(x, y, z) - Euler angles?
    // Let's implement it by setting the target based on rotation from eye.
    let r = renderer.clone();
    camera_obj.insert("set_rotation".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [_, Value::Number(_pitch), Value::Number(_yaw), Value::Number(_roll)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            let cam = ren.camera_mut();
            
            // Convert degrees to radians
            let p = (*_pitch as f32).to_radians();
            let y = (*_yaw as f32).to_radians();
            // roll is harder for a look_at camera without changing 'up', ignoring for now or just set up?
            
            let rotation = ferrous_engine::glam::Quat::from_euler(
                ferrous_engine::glam::EulerRot::YXZ,
                y, p, 0.0
            );
            let forward = rotation * Vec3::NEG_Z;
            cam.target = cam.eye + forward;
            
            Ok(Value::Number(0.0))
        } else {
            Err("Camera.set_rotation(pitch, yaw, roll) expected".into())
        }
    })));

    // Camera.set_mode_2d(width, height)
    let r = renderer.clone();
    camera_obj.insert("set_mode_2d".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
        if let [_, Value::Number(w), Value::Number(h)] = args.as_slice() {
            let mut ren = r.borrow_mut();
            ren.camera_mut().set_mode_2d(*w as f32, *h as f32);
            Ok(Value::Number(0.0))
        } else {
            Err("Camera.set_mode_2d(width, height) expected".into())
        }
    })));

    env.define("Camera".to_string(), Value::Object(camera_obj));
}
