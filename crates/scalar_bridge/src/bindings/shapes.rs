//! # 2D Shape Bindings for Scalar
//!
//! Pure2D coordinate system:
//! - Origin (0, 0) = center of the screen
//! - 1 unit = 1 pixel exactly
//! - X-axis: positive → right, negative → left
//! - Y-axis: positive → up, negative → down
//!
//! ## Exposed Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `Line(x1, y1, x2, y2 [, ...])` | 2D line segment |
//! | `Rect(x, y, width, height [, ...])` | 2D filled rectangle |
//! | `Circle(x, y, radius [, ...])` | 2D filled circle |
//! | `Triangle(x, y, size [, ...])` | Equilateral triangle |
//! | `Star(x, y, outer_r, inner_r, points [, ...])` | Multi-pointed star |
//! | `RegularPolygon(x, y, radius, sides [, ...])` | Regular polygon |
//! | `Polygon([points...] [, ...])` | Arbitrary polygon from point list |
//! | `SVG("path_data" [, ...])` | SVG path string rendering |
//!
//! ## Unified Kwarg Reference (all shapes except Line)
//!
//! | Kwarg | Type | Default | Description |
//! |-------|------|---------|-------------|
//! | `fill` | [r,g,b,a] | `[1,1,1,1]` (white) | Fill color |
//! | `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
//! | `stroke_width` | Number | `2.0` | Stroke thickness in pixels |
//! | `opacity` | Number | `1.0` | Global opacity (0.0–1.0) |
//! | `z_index` | Number | `0` | Z-order (higher = on top) |
//! | `rotation` | Number | `0` | Rotation in degrees (counter-clockwise) |
//! | `visible` | Boolean | `true` | Visibility |

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::{Renderer, Transform};
use ferrous_core::scene::world::types::{PathData, PathCommand};
use ferrous_engine::glam::{Vec2, Vec3, Quat};
use crate::LineData;

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub(super) fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => *n,
        _ => 0.0,
    }
}

/// Extracts [r, g, b, a] from args starting at `start`.
/// - If args[start] is a List → treat as [r, g, b, a?]
/// - If args are Numbers → r=args[start], g=args[start+1], b=args[start+2], a=args[start+3]?
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

/// Extracts [r, g, b, a] from a Value::List.
fn color_from_list(list: &[Value]) -> [f32; 4] {
    [
        num(list.get(0)) as f32,
        num(list.get(1)) as f32,
        num(list.get(2)) as f32,
        if list.len() >= 4 { num(list.get(3)) as f32 } else { 1.0 },
    ]
}

/// Resolves a color from kwargs with the given key, or returns default.
fn kwarg_color(kwargs: &HashMap<String, Value>, key: &str, default: [f32; 4]) -> [f32; 4] {
    kwargs.get(key).and_then(|v| match v {
        Value::List(list) if list.is_empty() => None,
        Value::List(list) => Some(color_from_list(list)),
        _ => None,
    }).unwrap_or(default)
}

/// Resolves a number from kwargs.
pub(super) fn kwarg_num(kwargs: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
    match kwargs.get(key) {
        Some(Value::Number(n)) => *n,
        _ => default,
    }
}

/// Resolves a boolean from kwargs.
fn kwarg_bool(kwargs: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    match kwargs.get(key) {
        Some(Value::Boolean(b)) => *b,
        _ => default,
    }
}

// ─── Unified shape spawner ────────────────────────────────────────────────────
//
// All shapes go through this helper: it builds a path, spawns it via
// spawn_2d_path, and then applies fill, stroke, opacity, z-index, rotation,
// and visibility from kwargs.

pub(super) struct ShapeKwargs {
    fill: Option<[f32; 4]>,
    stroke: Option<[f32; 4]>,
    stroke_width: f32,
    opacity: f32,
    z_index: i32,
    rotation_deg: f32,
    visible: bool,
    cap: u8,
}

pub(super) fn parse_shape_kwargs(kwargs: &HashMap<String, Value>) -> ShapeKwargs {
    // Parse fill: try "fill" kwarg first, then "fill_color" for backward compat.
    // If the selected kwarg is an empty list [] → explicit "no fill" (None).
    // If neither kwarg is present → default to white [1,1,1,1].
    fn parse_fill(kwargs: &HashMap<String, Value>) -> Option<[f32; 4]> {
        // Try "fill" first
        if let Some(val) = kwargs.get("fill") {
            match val {
                Value::List(list) if list.is_empty() => return None,
                Value::List(list) => return Some(color_from_list(list)),
                _ => return Some([1.0, 1.0, 1.0, 1.0]),
            }
        }
        // Fallback to "fill_color"
        if let Some(val) = kwargs.get("fill_color") {
            match val {
                Value::List(list) if list.is_empty() => return None,
                Value::List(list) => return Some(color_from_list(list)),
                _ => return Some([1.0, 1.0, 1.0, 1.0]),
            }
        }
        // No fill kwarg at all → default white
        Some([1.0, 1.0, 1.0, 1.0])
    }

    let fill = parse_fill(kwargs);

    let stroke = {
        let s = kwarg_color(kwargs, "stroke", [f32::NAN; 4]);
        if s[0].is_nan() { None } else { Some(s) }
    };

    let stroke_width = kwarg_num(kwargs, "stroke_width", 2.0) as f32;
    let opacity = kwarg_num(kwargs, "opacity", 1.0).clamp(0.0, 1.0) as f32;
    let z_index = kwarg_num(kwargs, "z_index", 0.0) as i32;
    let rotation_deg = kwarg_num(kwargs, "rotation", 0.0) as f32;
    let visible = kwarg_bool(kwargs, "visible", true);

    let cap: u8 = match kwargs.get("cap") {
        Some(Value::String(s)) if s == "square" => 2,
        Some(Value::String(s)) if s == "flat" => 0,
        _ => 1, // default round
    };

    ShapeKwargs { fill, stroke, stroke_width, opacity, z_index, rotation_deg, visible, cap }
}

/// Spawns a 2D shape from path commands, applies all kwargs, returns NodeId.
pub(super) fn spawn_2d_shape_with_kwargs(
    r: &mut Renderer,
    x: f32, y: f32,
    commands: Vec<PathCommand>,
    kwargs: &ShapeKwargs,
) -> u32 {
    let rot_rad = kwargs.rotation_deg * std::f32::consts::PI / 180.0;
    let transform = Transform::from_position(Vec3::new(x, y, 0.0))
        .with_rotation(Quat::from_rotation_z(rot_rad));

    let path = PathData { commands };
    let id = r.spawn_2d_path(transform, path);

    // Apply fill (default white). If fill is None, no fill is set (stroke-only).
    if let Some(fill) = kwargs.fill {
        let fill_color = [fill[0], fill[1], fill[2], fill[3] * kwargs.opacity];
        let _ = r.set_fill(id, fill_color);
    } else {
        let _ = r.remove_fill(id);
    }

    // Apply stroke
    if let Some(stroke) = kwargs.stroke {
        let stroke_color = [stroke[0], stroke[1], stroke[2], stroke[3] * kwargs.opacity];
        let _ = r.set_stroke(id, stroke_color, kwargs.stroke_width);
        let _ = r.set_line_cap(id, kwargs.cap);
    }

    let _ = r.set_z_index(id, kwargs.z_index);
    if !kwargs.visible {
        let _ = r.set_visible(id, false);
    }

    id.0 as u32
}

// ─── Path builders ────────────────────────────────────────────────────────────

/// Builds a rectangle path (centered at origin).
fn rect_path(width: f32, height: f32) -> Vec<PathCommand> {
    let hw = width * 0.5;
    let hh = height * 0.5;
    vec![
        PathCommand::MoveTo(Vec2::new(-hw, -hh)),
        PathCommand::LineTo(Vec2::new(hw, -hh)),
        PathCommand::LineTo(Vec2::new(hw, hh)),
        PathCommand::LineTo(Vec2::new(-hw, hh)),
        PathCommand::Close,
    ]
}

/// Builds a circle path (centered at origin) using 4 cubic beziers.
fn circle_path(radius: f32) -> Vec<PathCommand> {
    let c = 0.552284749831 * radius;
    vec![
        PathCommand::MoveTo(Vec2::new(radius, 0.0)),
        PathCommand::CubicTo(Vec2::new(radius, c), Vec2::new(c, radius), Vec2::new(0.0, radius)),
        PathCommand::CubicTo(Vec2::new(-c, radius), Vec2::new(-radius, c), Vec2::new(-radius, 0.0)),
        PathCommand::CubicTo(Vec2::new(-radius, -c), Vec2::new(-c, -radius), Vec2::new(0.0, -radius)),
        PathCommand::CubicTo(Vec2::new(c, -radius), Vec2::new(radius, -c), Vec2::new(radius, 0.0)),
        PathCommand::Close,
    ]
}

/// Builds an equilateral triangle path (centered at origin, pointing up).
fn triangle_path(size: f32) -> Vec<PathCommand> {
    let r = size * 0.5;
    let h = r * (3.0_f32).sqrt() / 2.0; // height of equilateral triangle
    let cx = 0.0;
    let cy = -h / 3.0; // centroid at origin
    vec![
        PathCommand::MoveTo(Vec2::new(cx, cy + h * 2.0 / 3.0)),        // top
        PathCommand::LineTo(Vec2::new(cx - r, cy - h / 3.0)),          // bottom-left
        PathCommand::LineTo(Vec2::new(cx + r, cy - h / 3.0)),          // bottom-right
        PathCommand::Close,
    ]
}

/// Builds a regular polygon path (centered at origin).
fn regular_polygon_path(radius: f32, sides: u32) -> Vec<PathCommand> {
    let sides = sides.max(3);
    let mut cmds = Vec::with_capacity(sides as usize + 1);
    let angle_step = std::f32::consts::TAU / sides as f32;
    // Start at top
    let first = Vec2::new(0.0, radius);
    cmds.push(PathCommand::MoveTo(first));
    for i in 1..sides {
        let a = angle_step * i as f32 - std::f32::consts::FRAC_PI_2;
        cmds.push(PathCommand::LineTo(Vec2::new(a.cos() * radius, a.sin() * radius)));
    }
    cmds.push(PathCommand::Close);
    cmds
}

/// Builds a star path (centered at origin). `inner_ratio` = inner_radius / outer_radius.
fn star_path(outer_radius: f32, inner_radius: f32, points: u32) -> Vec<PathCommand> {
    let points = points.max(3);
    let total = points * 2;
    let mut cmds = Vec::with_capacity(total as usize + 1);
    let angle_step = std::f32::consts::TAU / total as f32;
    // Start at top (outer point)
    let start_angle = -std::f32::consts::FRAC_PI_2;
    let first = Vec2::new(start_angle.cos() * outer_radius, start_angle.sin() * outer_radius);
    cmds.push(PathCommand::MoveTo(first));
    for i in 1..total {
        let a = angle_step * i as f32 + start_angle;
        let r = if i % 2 == 0 { outer_radius } else { inner_radius };
        cmds.push(PathCommand::LineTo(Vec2::new(a.cos() * r, a.sin() * r)));
    }
    cmds.push(PathCommand::Close);
    cmds
}

/// Builds an arbitrary polygon path from a list of points (centroid at origin).
fn polygon_path(points: &[(f32, f32)]) -> Vec<PathCommand> {
    if points.is_empty() {
        return vec![];
    }
    let mut cmds = Vec::with_capacity(points.len() + 1);
    cmds.push(PathCommand::MoveTo(Vec2::new(points[0].0, points[0].1)));
    for &(px, py) in &points[1..] {
        cmds.push(PathCommand::LineTo(Vec2::new(px, py)));
    }
    cmds.push(PathCommand::Close);
    cmds
}

// ─── SVG Path Parser ──────────────────────────────────────────────────────────
//
// Parses a subset of SVG path data:
//   M x y     — move to (absolute)
//   m dx dy   — move to (relative)
//   L x y     — line to (absolute)
//   l dx dy   — line to (relative)
//   C x1 y1 x2 y2 x y — cubic bezier (absolute)
//   c dx1 dy1 dx2 dy2 dx dy — cubic bezier (relative)
//   Q x1 y1 x y — quadratic bezier (absolute, converted to cubic)
//   q dx1 dy1 dx dy — quadratic bezier (relative)
//   Z / z     — close path
//   H x       — horizontal line (absolute)
//   h dx      — horizontal line (relative)
//   V y       — vertical line (absolute)
//   v dy      — vertical line (relative)
//   S x2 y2 x y — smooth cubic bezier (absolute)
//   s dx2 dy2 dx dy — smooth cubic bezier (relative)
//   T x y     — smooth quadratic bezier (absolute)
//   t dx dy   — smooth quadratic bezier (relative)

#[derive(Default)]
struct SvgParser {
    cx: f32,   // current point x
    cy: f32,   // current point y
    scx: f32,  // last smooth control point x
    scy: f32,  // last smooth control point y
    first_x: f32,
    first_y: f32,
}

pub(super) fn parse_svg_path(svg: &str) -> Result<Vec<PathCommand>, String> {
    let mut parser = SvgParser::default();
    let mut cmds: Vec<PathCommand> = Vec::new();

    // Tokenize: letters are commands, numbers are arguments
    let mut chars = svg.chars().peekable();
    let mut buf = String::new();

    fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, buf: &mut String) -> Option<f32> {
        buf.clear();
        // Skip whitespace and commas
        while let Some(&c) = chars.peek() {
            if c == ' ' || c == ',' || c == '\t' || c == '\n' || c == '\r' {
                chars.next();
            } else {
                break;
            }
        }
        // Read sign
        if let Some(&c) = chars.peek() {
            if c == '-' || c == '+' {
                buf.push(c);
                chars.next();
            }
        }
        // Read digits and decimal point (mantissa)
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
        }
        // Read exponent part (e.g. e10, e-5, E+3)
        if let Some(&c) = chars.peek() {
            if c == 'e' || c == 'E' {
                buf.push(c);
                chars.next();
                // Optional exponent sign
                if let Some(&c) = chars.peek() {
                    if c == '-' || c == '+' {
                        buf.push(c);
                        chars.next();
                    }
                }
                // Exponent digits
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        buf.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        }
        if buf.is_empty() || buf == "-" || buf == "+" || buf == "." || buf == "-." || buf == "+." {
            None
        } else {
            buf.parse::<f32>().ok()
        }
    }

    fn read_point(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, buf: &mut String) -> Option<(f32, f32)> {
        let x = read_number(chars, buf)?;
        let y = read_number(chars, buf)?;
        Some((x, y))
    }

    loop {
        // Skip whitespace before command
        while let Some(&c) = chars.peek() {
            if c == ' ' || c == ',' || c == '\t' || c == '\n' || c == '\r' {
                chars.next();
            } else {
                break;
            }
        }

        let cmd = match chars.next() {
            None => break,
            Some(c) => c,
        };

        match cmd {
            'M' | 'm' => {
                let rel = cmd == 'm';
                if let Some((x, y)) = read_point(&mut chars, &mut buf) {
                    let (dx, dy) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    cmds.push(PathCommand::MoveTo(Vec2::new(dx, dy)));
                    parser.cx = dx;
                    parser.cy = dy;
                    parser.first_x = dx;
                    parser.first_y = dy;
                    // Subsequent coordinate pairs after M are treated as implicit L
                    parser.scx = parser.cx;
                    parser.scy = parser.cy;
                    while let Some((x, y)) = read_point(&mut chars, &mut buf) {
                        let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                        cmds.push(PathCommand::LineTo(Vec2::new(nx, ny)));
                        parser.cx = nx;
                        parser.cy = ny;
                        parser.scx = parser.cx;
                        parser.scy = parser.cy;
                    }
                }
            }
            'L' | 'l' => {
                let rel = cmd == 'l';
                while let Some((x, y)) = read_point(&mut chars, &mut buf) {
                    let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    cmds.push(PathCommand::LineTo(Vec2::new(nx, ny)));
                    parser.cx = nx;
                    parser.cy = ny;
                    parser.scx = parser.cx;
                    parser.scy = parser.cy;
                }
            }
            'H' | 'h' => {
                let rel = cmd == 'h';
                while let Some(x) = read_number(&mut chars, &mut buf) {
                    let nx = if rel { parser.cx + x } else { x };
                    cmds.push(PathCommand::LineTo(Vec2::new(nx, parser.cy)));
                    parser.cx = nx;
                    parser.scx = parser.cx;
                }
            }
            'V' | 'v' => {
                let rel = cmd == 'v';
                while let Some(y) = read_number(&mut chars, &mut buf) {
                    let ny = if rel { parser.cy + y } else { y };
                    cmds.push(PathCommand::LineTo(Vec2::new(parser.cx, ny)));
                    parser.cy = ny;
                    parser.scy = parser.cy;
                }
            }
            'C' | 'c' => {
                let rel = cmd == 'c';
                while let (Some((x1, y1)), Some((x2, y2)), Some((x, y))) = (
                    read_point(&mut chars, &mut buf),
                    read_point(&mut chars, &mut buf),
                    read_point(&mut chars, &mut buf),
                ) {
                    let (cx1, cy1) = if rel { (parser.cx + x1, parser.cy + y1) } else { (x1, y1) };
                    let (cx2, cy2) = if rel { (parser.cx + x2, parser.cy + y2) } else { (x2, y2) };
                    let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    cmds.push(PathCommand::CubicTo(Vec2::new(cx1, cy1), Vec2::new(cx2, cy2), Vec2::new(nx, ny)));
                    parser.scx = cx2;
                    parser.scy = cy2;
                    parser.cx = nx;
                    parser.cy = ny;
                }
            }
            'S' | 's' => {
                let rel = cmd == 's';
                while let (Some((x2, y2)), Some((x, y))) = (
                    read_point(&mut chars, &mut buf),
                    read_point(&mut chars, &mut buf),
                ) {
                    // Reflection of last control point
                    let cx1 = parser.cx + (parser.cx - parser.scx);
                    let cy1 = parser.cy + (parser.cy - parser.scy);
                    let (cx2, cy2) = if rel { (parser.cx + x2, parser.cy + y2) } else { (x2, y2) };
                    let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    cmds.push(PathCommand::CubicTo(Vec2::new(cx1, cy1), Vec2::new(cx2, cy2), Vec2::new(nx, ny)));
                    parser.scx = cx2;
                    parser.scy = cy2;
                    parser.cx = nx;
                    parser.cy = ny;
                }
            }
            'Q' | 'q' => {
                let rel = cmd == 'q';
                while let (Some((x1, y1)), Some((x, y))) = (
                    read_point(&mut chars, &mut buf),
                    read_point(&mut chars, &mut buf),
                ) {
                    let (cqx, cqy) = if rel { (parser.cx + x1, parser.cy + y1) } else { (x1, y1) };
                    let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    // Convert quadratic to cubic: CP1 = Q0 + 2/3*(Q1-Q0), CP2 = Q2 + 2/3*(Q1-Q2)
                    let cx1 = parser.cx + (2.0 / 3.0) * (cqx - parser.cx);
                    let cy1 = parser.cy + (2.0 / 3.0) * (cqy - parser.cy);
                    let cx2 = nx + (2.0 / 3.0) * (cqx - nx);
                    let cy2 = ny + (2.0 / 3.0) * (cqy - ny);
                    cmds.push(PathCommand::CubicTo(Vec2::new(cx1, cy1), Vec2::new(cx2, cy2), Vec2::new(nx, ny)));
                    parser.scx = cqx;
                    parser.scy = cqy;
                    parser.cx = nx;
                    parser.cy = ny;
                }
            }
            'T' | 't' => {
                let rel = cmd == 't';
                while let Some((x, y)) = read_point(&mut chars, &mut buf) {
                    // Reflection of last quadratic control point
                    let cqx = parser.cx + (parser.cx - parser.scx);
                    let cqy = parser.cy + (parser.cy - parser.scy);
                    let (nx, ny) = if rel { (parser.cx + x, parser.cy + y) } else { (x, y) };
                    let cx1 = parser.cx + (2.0 / 3.0) * (cqx - parser.cx);
                    let cy1 = parser.cy + (2.0 / 3.0) * (cqy - parser.cy);
                    let cx2 = nx + (2.0 / 3.0) * (cqx - nx);
                    let cy2 = ny + (2.0 / 3.0) * (cqy - ny);
                    cmds.push(PathCommand::CubicTo(Vec2::new(cx1, cy1), Vec2::new(cx2, cy2), Vec2::new(nx, ny)));
                    parser.scx = cqx;
                    parser.scy = cqy;
                    parser.cx = nx;
                    parser.cy = ny;
                }
            }
            'Z' | 'z' => {
                cmds.push(PathCommand::Close);
                parser.cx = parser.first_x;
                parser.cy = parser.first_y;
                parser.scx = parser.cx;
                parser.scy = parser.cy;
            }
            // Implicit commands after numbers (e.g., "M 10 10 20 20" where 20,20 is implicit L)
            _ => return Err(format!("Unknown SVG command: '{}'", cmd)),
        }
    }

    Ok(cmds)
}

// ─── Public registration ───────────────────────────────────────────────────────

pub fn register(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, line_data: Rc<RefCell<HashMap<u64, LineData>>>) {
    register_line(env, renderer.clone(), line_data);
    register_rect(env, renderer.clone());
    register_circle(env, renderer.clone());
    register_triangle(env, renderer.clone());
    register_star(env, renderer.clone());
    register_regular_polygon(env, renderer.clone());
    register_polygon(env, renderer.clone());
    register_svg(env, renderer.clone());
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

            let cap_style: u8 = match kwargs.get("cap") {
                Some(Value::String(s)) if s == "square" => 2,
                Some(Value::String(s)) if s == "flat" => 0,
                _ => 1,
            };

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
                kwarg_num(&kwargs, "stroke_width", 1.0) as f32
            };

            let color = if args.len() > next_arg {
                extract_color(&args, next_arg)
            } else {
                kwarg_color(&kwargs, "stroke", [1.0, 1.0, 1.0, 1.0])
            };

            let opacity = kwarg_num(&kwargs, "opacity", 1.0).clamp(0.0, 1.0) as f32;
            let z_index = kwarg_num(&kwargs, "z_index", 0.0) as i32;
            let visible = kwarg_bool(&kwargs, "visible", true);

            let mut r = ren.borrow_mut();
            let node_id = r.spawn_2d_line(x1, y1, x2, y2, thickness);
            let _ = r.set_stroke(node_id, [color[0], color[1], color[2], color[3] * opacity], thickness);
            let _ = r.set_line_cap(node_id, cap_style);
            let _ = r.set_z_index(node_id, z_index);
            if !visible {
                let _ = r.set_visible(node_id, false);
            }

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
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 4 {
                return Err("Rect(x, y, width, height [, color...]) needs 4+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let w = num(args.get(2)) as f32;
            let h = num(args.get(3)) as f32;

            let sk = parse_shape_kwargs(&kwargs);

            // Allow color via positional args for backward compatibility (overrides fill kwarg)
            let sk = if args.len() > 4 {
                let pos_color = extract_color(&args, 4);
                ShapeKwargs { fill: Some(pos_color), ..sk }
            } else {
                sk
            };

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, rect_path(w, h), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── Circle ───────────────────────────────────────────────────────────────────

fn register_circle(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Circle".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 3 {
                return Err("Circle(x, y, radius [, color...]) needs 3+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let radius = num(args.get(2)) as f32;

            let sk = parse_shape_kwargs(&kwargs);

            // Allow color via positional args for backward compatibility (overrides fill kwarg)
            let sk = if args.len() > 3 {
                let pos_color = extract_color(&args, 3);
                ShapeKwargs { fill: Some(pos_color), ..sk }
            } else {
                sk
            };

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, circle_path(radius), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── Triangle ─────────────────────────────────────────────────────────────────

fn register_triangle(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Triangle".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 3 {
                return Err("Triangle(x, y, size [, kwargs]) needs 3+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let size = num(args.get(2)) as f32;

            let sk = parse_shape_kwargs(&kwargs);

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, triangle_path(size), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── Star ─────────────────────────────────────────────────────────────────────

fn register_star(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Star".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 5 {
                return Err("Star(x, y, outer_r, inner_r, points [, kwargs]) needs 5+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let outer_r = num(args.get(2)) as f32;
            let inner_r = num(args.get(3)) as f32;
            let points = num(args.get(4)) as u32;

            let sk = parse_shape_kwargs(&kwargs);

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, star_path(outer_r, inner_r, points), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── RegularPolygon ──────────────────────────────────────────────────────────

fn register_regular_polygon(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "RegularPolygon".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.len() < 4 {
                return Err("RegularPolygon(x, y, radius, sides [, kwargs]) needs 4+ args".to_string());
            }

            let x = num(args.get(0)) as f32;
            let y = num(args.get(1)) as f32;
            let radius = num(args.get(2)) as f32;
            let sides = num(args.get(3)) as u32;

            let sk = parse_shape_kwargs(&kwargs);

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, regular_polygon_path(radius, sides), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── Polygon ─────────────────────────────────────────────────────────────────

fn register_polygon(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "Polygon".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            if args.is_empty() {
                return Err("Polygon([point, ...] [, kwargs]) needs at least one point list".to_string());
            }

            // Parse points from first arg (list of [x,y] pairs)
            let points = match &args[0] {
                Value::List(list) => {
                    let mut pts = Vec::with_capacity(list.len());
                    for item in list {
                        match item {
                            Value::List(p) if p.len() >= 2 => {
                                pts.push((num(Some(&p[0])) as f32, num(Some(&p[1])) as f32));
                            }
                            _ => return Err("Polygon: each point must be [x, y]".to_string()),
                        }
                    }
                    if pts.len() < 3 {
                        return Err("Polygon: need at least 3 points".to_string());
                    }
                    pts
                }
                _ => return Err("Polygon: first argument must be a list of [x,y] points".to_string()),
            };

            // Compute centroid for centering the transform
            let cx = points.iter().map(|p| p.0).sum::<f32>() / points.len() as f32;
            let cy = points.iter().map(|p| p.1).sum::<f32>() / points.len() as f32;

            // Offset points so centroid is at origin (the transform handles positioning)
            let centered: Vec<(f32, f32)> = points.iter().map(|(px, py)| (px - cx, py - cy)).collect();

            let sk = parse_shape_kwargs(&kwargs);

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, cx, cy, polygon_path(&centered), &sk);
            Ok(Value::NodeId(id))
        })),
    );
}

// ─── SVG path rendering ──────────────────────────────────────────────────────

fn register_svg(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "SVG".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let path_str = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("SVG(\"path_data\" [, kwargs]): first argument must be a string".to_string()),
            };

            let x = kwarg_num(&kwargs, "x", 0.0) as f32;
            let y = kwarg_num(&kwargs, "y", 0.0) as f32;
            let scale = kwarg_num(&kwargs, "scale", 1.0) as f32;

            let mut cmds = parse_svg_path(&path_str)?;

            // Apply scale to all coordinates
            if (scale - 1.0).abs() > f32::EPSILON {
                for cmd in &mut cmds {
                    match cmd {
                        PathCommand::MoveTo(p) => {
                            *p = Vec2::new(p.x * scale, p.y * scale);
                        }
                        PathCommand::LineTo(p) => {
                            *p = Vec2::new(p.x * scale, p.y * scale);
                        }
                        PathCommand::CubicTo(c1, c2, p) => {
                            *c1 = Vec2::new(c1.x * scale, c1.y * scale);
                            *c2 = Vec2::new(c2.x * scale, c2.y * scale);
                            *p = Vec2::new(p.x * scale, p.y * scale);
                        }
                        PathCommand::Close => {}
                    }
                }
            }

            let sk = parse_shape_kwargs(&kwargs);

            let mut r = ren.borrow_mut();
            let id = spawn_2d_shape_with_kwargs(&mut r, x, y, cmds, &sk);
            Ok(Value::NodeId(id))
        })),
    );
}
