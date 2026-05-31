//! # Bindings de formas 2D para Scalar
//!
//! Sistema de coordenadas Pure2D:
//! - Origen (0, 0) = centro de la pantalla
//! - 1 unidad = 1 píxel exacto
//! - Eje X: positivo → derecha, negativo → izquierda
//! - Eje Y: positivo → arriba, negativo → abajo
//!
//! ## Funciones expuestas
//!
//! ### Líneas
//! - `Line(x1, y1, x2, y2)` — línea blanca de 1px
//! - `Line(x1, y1, x2, y2, thickness)` — grosor personalizado
//! - `Line(x1, y1, x2, y2, thickness, r, g, b, a)` — con color
//! - `Line([x1, y1], [x2, y2], thickness, color)` — sintaxis punto
//!
//! ### Rectángulos
//! - `Rect(x, y, width, height)` — rectángulo blanco centrado en (x,y)
//! - `Rect(x, y, width, height, r, g, b, a)` — con color
//!
//! ### Círculos  
//! - `Circle(x, y, radius)` — círculo blanco centrado en (x,y)
//! - `Circle(x, y, radius, r, g, b, a)` — con color

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::Renderer;
use crate::LineData;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => *n,
        _ => 0.0,
    }
}

/// Extrae [r, g, b, a] desde args[start].
/// - Si args[start] es una List → tomar como [r, g, b, a?]
/// - Si son Number → r=args[start], g=args[start+1], b=args[start+2], a=args[start+3]?
fn extract_color(args: &[Value], start: usize) -> [f32; 4] {
    if start >= args.len() {
        return [1.0, 1.0, 1.0, 1.0];
    }
    if let Value::List(list) = &args[start] {
        let r = num(list.get(0)) as f32;
        let g = num(list.get(1)) as f32;
        let b = num(list.get(2)) as f32;
        let a = if list.len() >= 4 { num(list.get(3)) as f32 } else { 1.0 };
        return [r, g, b, a];
    }
    let r = num(args.get(start)) as f32;
    let g = num(args.get(start + 1)) as f32;
    let b = num(args.get(start + 2)) as f32;
    let a = if args.len() > start + 3 { num(args.get(start + 3)) as f32 } else { 1.0 };
    [r, g, b, a]
}

// ─── Public registration ───────────────────────────────────────────────────────

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, line_data: Rc<RefCell<HashMap<u64, LineData>>>) {
    register_line(env, renderer.clone(), line_data);
    register_rect(env, renderer.clone());
    register_circle(env, renderer.clone());
}

// ─── Line ─────────────────────────────────────────────────────────────────────
fn register_line(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, line_data: Rc<RefCell<HashMap<u64, LineData>>>) {
    let ren = renderer.clone();
    let ld = line_data.clone();
    env.define(
        "Line".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 2 {
                return Err("Line requires at least 2 arguments".to_string());
            }

            // Parse cap style from kwargs: cap: "round" / "square" / "flat" (default 1 = round)
            let cap_style: u8 = match kwargs.get("cap") {
                Some(Value::String(s)) if s == "round" => 1,
                Some(Value::String(s)) if s == "square" => 2,
                Some(Value::String(s)) if s == "flat" => 0,
                Some(Value::Number(n)) => n.clamp(0.0, 2.0) as u8,
                _ => 1, // default: round
            };

            // Parse endpoints — soporta tanto (x1,y1,x2,y2) como ([x1,y1],[x2,y2])
            let (x1, y1, x2, y2, mut next_arg) = match (&args[0], &args[1]) {
                (Value::List(p1), Value::List(p2)) if p1.len() >= 2 && p2.len() >= 2 => (
                    num(Some(&p1[0])) as f32, num(Some(&p1[1])) as f32,
                    num(Some(&p2[0])) as f32, num(Some(&p2[1])) as f32, 2,
                ),
                (Value::Number(_), Value::Number(_)) if args.len() >= 4 => (
                    num(args.get(0)) as f32, num(args.get(1)) as f32,
                    num(args.get(2)) as f32, num(args.get(3)) as f32, 4,
                ),
                _ => return Err("Line: use Line(x1,y1,x2,y2) or Line([x1,y1],[x2,y2])".to_string()),
            };

            let thickness = if args.len() > next_arg {
                let t = num(args.get(next_arg)) as f32;
                next_arg += 1;
                t
            } else {
                1.0
            };

            let color = if args.len() > next_arg {
                extract_color(&args, next_arg)
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };

            let mut r = ren.borrow_mut();
            let node_id = r.spawn_2d_line(x1, y1, x2, y2, thickness);
            let _ = r.set_stroke(node_id, color, thickness);
            let _ = r.set_line_cap(node_id, cap_style);

            // Store original endpoints for animation tracking
            ld.borrow_mut().insert(node_id.0, LineData { x1, y1, x2, y2 });

            Ok(Value::NodeId(node_id.0 as u32))
        })),
    );
}

// ─── Rect ─────────────────────────────────────────────────────────────────────
fn register_rect(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Rect".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            if args.len() < 4 {
                return Err("Rect(x, y, width, height [, color...]) needs 4+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let w = num(args.get(2)) as f32;
            let h = num(args.get(3)) as f32;
            let color = extract_color(&args, 4);

            let mut r = ren.borrow_mut();
            // z = 0 por defecto; centro en (x, y)
            let node_id = r.spawn_2d_rect(x, y, 0.0, w, h);
            let _ = r.set_fill(node_id, color);

            Ok(Value::NodeId(node_id.0 as u32))
        })),
    );
}

// ─── Circle ───────────────────────────────────────────────────────────────────
fn register_circle(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Circle".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            if args.len() < 3 {
                return Err("Circle(x, y, radius [, color...]) needs 3+ args".to_string());
            }

            let x      = num(args.get(0)) as f32;
            let y      = num(args.get(1)) as f32;
            let radius = num(args.get(2)) as f32;
            let color  = extract_color(&args, 3);

            let mut r = ren.borrow_mut();
            let node_id = r.spawn_2d_circle(x, y, 0.0, radius);
            let _ = r.set_fill(node_id, color);

            Ok(Value::NodeId(node_id.0 as u32))
        })),
    );
}
