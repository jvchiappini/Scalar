//! # File Import & Text Rendering Bindings for Scalar
//!
//! Exposed functions:
//!
//! | Function | Description |
//! |----------|-------------|
//! | `SVGImport(path)` | Load an SVG file and render all `<path>` elements |
//! | `FontImport(path)` | Load a TrueType/OpenType font for text rendering |
//! | `Text(str, x, y [, ...kwargs])` | Render a string as vector paths using a loaded font |
//!
//! ## Text Kwarg Reference
//!
//! | Kwarg | Type | Default | Description |
//! |-------|------|---------|-------------|
//! | `font` | Number | `0` | Index of the loaded font (returned by `FontImport`) |
//! | `size` | Number | `48` | Font size in pixels |
//! | `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
//! | `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
//! | `stroke_width` | Number | `2.0` | Stroke thickness |
//! | `opacity` | Number | `1.0` | Global opacity |
//! | `rotation` | Number | `0` | Rotation in degrees |
//! | `z_index` | Number | `0` | Z-order |
//! | `visible` | Boolean | `true` | Visibility |

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scalar_lang::{Environment, Value};
use ferrous_engine::Renderer;
use ferrous_engine::glam::Vec2;
use ferrous_core::scene::world::types::PathCommand;
use crate::bindings::shapes::{self, num, kwarg_num};

/// Internal: stores raw TTF/OTF bytes for text-to-path rendering via ttf-parser.
pub struct FontEntry {
    pub bytes: Vec<u8>,
}

/// Per-character path data returned by `glyph_paths_for_text`.
pub struct GlyphPathData {
    pub commands: Vec<PathCommand>,
    pub width: f32,
    pub character: char,
}

/// Registers SVGImport, FontImport, and Text into the Scalar environment.
pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    fonts: Rc<RefCell<Vec<FontEntry>>>,
) {
    register_svg_import(env, renderer.clone());
    register_font_import(env, fonts.clone());
    register_text(env, renderer.clone(), fonts);
}

// ─── SVG Import ───────────────────────────────────────────────────────────────
//
// Parses a subset of SVG XML:
//   - Extracts <path> elements
//   - Reads d, fill, stroke, stroke-width, opacity, transform attributes
//   - Supports #rgb, #rrggbb, named colors, and "none"
//   - Delegates path string parsing to shapes::parse_svg_path

fn register_svg_import(env: &mut Environment, renderer: Rc<RefCell<Renderer>>) {
    let ren = renderer.clone();
    env.define(
        "SVGImport".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            let path = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("SVGImport(path): first argument must be a string path".to_string()),
            };

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => return Err(format!("SVGImport: failed to read '{}': {}", path, e)),
            };

            let paths = match extract_svg_paths(&content) {
                Ok(p) => p,
                Err(e) => return Err(format!("SVGImport: {}: {}", path, e)),
            };

            if paths.is_empty() {
                return Err(format!("SVGImport: no <path> elements found in '{}'", path));
            }

            let mut r = ren.borrow_mut();
            let mut node_ids = Vec::with_capacity(paths.len());

            for svg_path in &paths {
                let cmds = match shapes::parse_svg_path(&svg_path.d) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("SVGImport: skipping a <path> due to parse error: {}", e);
                        continue;
                    }
                };

                // Build kwargs from SVG attributes where present
                let mut kwargs = HashMap::new();

                if let Some(fill) = &svg_path.fill {
                    if let Some(color_val) = svg_color_to_value(fill) {
                        kwargs.insert("fill".to_string(), color_val);
                    }
                }
                if let Some(stroke) = &svg_path.stroke {
                    if let Some(color_val) = svg_color_to_value(stroke) {
                        kwargs.insert("stroke".to_string(), color_val);
                    }
                }
                if let Some(sw) = svg_path.stroke_width {
                    kwargs.insert("stroke_width".to_string(), Value::Number(sw as f64));
                }
                if let Some(op) = svg_path.opacity {
                    kwargs.insert("opacity".to_string(), Value::Number(op as f64));
                }
                if let Some(z) = svg_path.z_index {
                    kwargs.insert("z_index".to_string(), Value::Number(z as f64));
                }

                let sk = shapes::parse_shape_kwargs(&kwargs);
                let id = shapes::spawn_2d_shape_with_kwargs(&mut r, 0.0, 0.0, cmds, &sk);
                node_ids.push(Value::NodeId(id));
            }

            Ok(Value::List(node_ids))
        })),
    );
}

/// Parsed SVG path element data.
struct SvgPathData {
    d: String,
    fill: Option<String>,
    stroke: Option<String>,
    stroke_width: Option<f32>,
    opacity: Option<f32>,
    z_index: Option<f32>,
}

/// Extracts <path ...> elements from SVG XML using a simple scanner.
/// Handles `d="..."`, `d='...'`, attributes in any order,
/// self-closing tags `<path/>`, and namespace prefixes like `<svg:path>`.
fn extract_svg_paths(xml: &str) -> Result<Vec<SvgPathData>, String> {
    let mut results = Vec::new();
    let mut pos = 0;
    let bytes = xml.as_bytes();

    loop {
        // Find the next <path ...> tag
        let tag_start = match find_subsequence(bytes, b"<path", pos)
            .or_else(|| find_subsequence(bytes, b"<svg:path", pos))
        {
            Some(p) => p,
            None => break,
        };

        // Find the closing '>'
        let tag_end = match find_byte(bytes, b'>', tag_start + 1) {
            Some(p) => p,
            None => return Err("SVGImport: unclosed <path> tag".to_string()),
        };

        let tag_content = &xml[tag_start..=tag_end];

        // Extract attributes
        let d = extract_attr(tag_content, "d").unwrap_or_default();
        let fill = extract_attr(tag_content, "fill");
        let stroke = extract_attr(tag_content, "stroke");
        let stroke_width = extract_attr(tag_content, "stroke-width")
            .and_then(|s| s.parse::<f32>().ok());
        let opacity = extract_attr(tag_content, "opacity")
            .and_then(|s| s.parse::<f32>().ok());
        let z_index = None;

        if !d.is_empty() {
            results.push(SvgPathData {
                d,
                fill,
                stroke,
                stroke_width,
                opacity,
                z_index,
            });
        }

        pos = tag_end + 1;
    }

    Ok(results)
}

/// Finds the first occurrence of `needle` in `haystack` starting from `start`.
fn find_subsequence(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|i| start + i)
}

/// Finds the first occurrence of byte `b` starting from `pos`.
fn find_byte(data: &[u8], b: u8, pos: usize) -> Option<usize> {
    data[pos..].iter().position(|&x| x == b).map(|i| pos + i)
}

/// Extracts an attribute value from an SVG tag string.
/// Handles `name="value"` and `name='value'` forms.
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    // Try double-quoted form: name="..."
    let double = format!("{}=\"", name);
    if let Some(start) = tag.find(&double) {
        let val_start = start + double.len();
        if let Some(end) = tag[val_start..].find('"') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    // Try single-quoted form: name='...'
    let single = format!("{}='", name);
    if let Some(start) = tag.find(&single) {
        let val_start = start + single.len();
        if let Some(end) = tag[val_start..].find('\'') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    None
}

/// Converts an SVG color string to a Scalar Value (list of [r,g,b,a]).
/// "none" returns an empty list [] (sentinel for "no fill/transparent").
/// Supports:
///   - "none" → [] (explicit no fill)
///   - "transparent" → [] 
///   - "#rgb" → #rrggbb shorthand
///   - "#rrggbb" / "#rrggbbaa"
///   - Named CSS colors (basic set)
fn svg_color_to_value(color: &str) -> Option<Value> {
    let color = color.trim();
    if color.eq_ignore_ascii_case("none") || color.eq_ignore_ascii_case("transparent") {
        return Some(Value::List(vec![]));  // sentinel for "no fill"
    }

    // Hex colors
    if let Some(hex) = color.strip_prefix('#') {
        let rgba = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
            }
            _ => return None,
        };
        return Some(Value::List(vec![
            Value::Number(rgba[0] as f64),
            Value::Number(rgba[1] as f64),
            Value::Number(rgba[2] as f64),
            Value::Number(rgba[3] as f64),
        ]));
    }

    // Named colors (CSS basic + extended common)
    let named: Option<[f32; 4]> = match color.to_lowercase().as_str() {
        "black" => Some([0.0, 0.0, 0.0, 1.0]),
        "white" => Some([1.0, 1.0, 1.0, 1.0]),
        "red" => Some([1.0, 0.0, 0.0, 1.0]),
        "green" | "lime" => Some([0.0, 1.0, 0.0, 1.0]),
        "blue" => Some([0.0, 0.0, 1.0, 1.0]),
        "yellow" => Some([1.0, 1.0, 0.0, 1.0]),
        "cyan" | "aqua" => Some([0.0, 1.0, 1.0, 1.0]),
        "magenta" | "fuchsia" => Some([1.0, 0.0, 1.0, 1.0]),
        "gray" | "grey" => Some([0.5, 0.5, 0.5, 1.0]),
        "silver" => Some([0.75, 0.75, 0.75, 1.0]),
        "maroon" => Some([0.5, 0.0, 0.0, 1.0]),
        "purple" => Some([0.5, 0.0, 0.5, 1.0]),
        "navy" => Some([0.0, 0.0, 0.5, 1.0]),
        "orange" => Some([1.0, 0.65, 0.0, 1.0]),
        "pink" => Some([1.0, 0.75, 0.8, 1.0]),
        "brown" => Some([0.65, 0.16, 0.16, 1.0]),
        _ => None,
    };

    named.map(|[r, g, b, a]| {
        Value::List(vec![
            Value::Number(r as f64),
            Value::Number(g as f64),
            Value::Number(b as f64),
            Value::Number(a as f64),
        ])
    })
}

// ─── Font Import ──────────────────────────────────────────────────────────────

fn register_font_import(env: &mut Environment, fonts: Rc<RefCell<Vec<FontEntry>>>) {
    env.define(
        "FontImport".to_string(),
        Value::NativeFunction(Rc::new(move |args, _kwargs: HashMap<String, Value>| {
            let path = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("FontImport(path): first argument must be a string path".to_string()),
            };

            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) => return Err(format!("FontImport: failed to read '{}': {}", path, e)),
            };

            // Quick sanity-check: ttf-parser will fail gracefully, but we want
            // a clear error message at import time.
            if ttf_parser::Face::parse(&bytes, 0).is_err() {
                return Err(format!("FontImport: '{}' is not a valid TTF/OTF font", path));
            }

            let mut font_list = fonts.borrow_mut();
            let index = font_list.len();
            font_list.push(FontEntry { bytes });

            Ok(Value::Number(index as f64))
        })),
    );
}

// ─── Text Rendering ───────────────────────────────────────────────────────────
//
// Renders a text string as vector paths using a previously loaded font.
// The text is a single combined path spanning multiple glyphs.
// Uses the same unified kwargs system as other shapes.

fn register_text(env: &mut Environment, renderer: Rc<RefCell<Renderer>>, fonts: Rc<RefCell<Vec<FontEntry>>>) {
    env.define(
        "Text".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            let text_str = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => return Err("Text(str, x, y [, kwargs]): first argument must be a string".to_string()),
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
            let font_size  = kwarg_num(&kwargs, "size", 48.0) as f32;

            let font_list = fonts.borrow();
            let font_bytes = match font_list.get(font_index) {
                Some(entry) => &entry.bytes,
                None => return Err(format!(
                    "Text: font index {} not found. Load a font first with FontImport(path). Loaded: {}",
                    font_index, font_list.len()
                )),
            };

            // Parse the face with the battle-tested ttf-parser crate.
            let face = match ttf_parser::Face::parse(font_bytes, 0) {
                Ok(f) => f,
                Err(e) => return Err(format!("Text: ttf-parser failed to load face: {:?}", e)),
            };

            let upem = face.units_per_em() as f32;
            let scale = font_size / upem;

            let mut combined_cmds: Vec<PathCommand> = Vec::new();
            let mut cursor_x: f32 = 0.0;

            for ch in text_str.chars() {
                let gid = match face.glyph_index(ch) {
                    Some(id) => id,
                    None => {
                        // Advance by the advance of the space glyph (or ~0.5em) and skip
                        if let Some(sid) = face.glyph_index(' ') {
                            let adv = face.glyph_hor_advance(sid).unwrap_or((upem * 0.5) as u16) as f32;
                            cursor_x += adv * scale;
                        } else {
                            cursor_x += font_size * 0.5;
                        }
                        continue;
                    }
                };

                let adv = face.glyph_hor_advance(gid).unwrap_or(0) as f32;

                // Build an outline for this glyph.
                let mut builder = GlyphPathBuilder {
                    cmds: Vec::new(),
                    cursor_x,
                    scale,
                    cur_x: 0.0,
                    cur_y: 0.0,
                };
                // outline_glyph returns None for glyphs with no outline (e.g. space)
                let _ = face.outline_glyph(gid, &mut builder);

                combined_cmds.extend(builder.cmds);
                cursor_x += adv * scale;
            }

            if combined_cmds.is_empty() {
                return Err("Text: no glyphs could be rendered (empty string or unsupported chars)".to_string());
            }

            let mut full_kwargs = kwargs.clone();
            full_kwargs.remove("font");
            full_kwargs.remove("size");
            full_kwargs.remove("x");
            full_kwargs.remove("y");

            let sk = shapes::parse_shape_kwargs(&full_kwargs);
            let mut r = renderer.borrow_mut();
            let id = shapes::spawn_2d_shape_with_kwargs(&mut r, x, y, combined_cmds, &sk);

            Ok(Value::NodeId(id))
        })),
    );
}

/// Builds a list of per-character path data from a text string using a loaded font.
///
/// Each entry contains the `PathCommand` sequence for one glyph, its advance width
/// in pixels, and the character itself.  This is used by `WriteText()` to animate
/// characters individually.
///
/// # Arguments
/// * `font_bytes` — Raw bytes of a TTF/OTF font file.
/// * `text` — The string to render.
/// * `font_size` — Font size in pixels.
///
/// # Returns
/// A `Vec<GlyphPathData>` with one entry per renderable character.  Characters that
/// have no outline (e.g. spaces) are skipped.
pub fn glyph_paths_for_text(
    font_bytes: &[u8],
    text: &str,
    font_size: f32,
) -> Result<Vec<GlyphPathData>, String> {
    let face = ttf_parser::Face::parse(font_bytes, 0)
        .map_err(|e| format!("ttf-parser failed to load face: {:?}", e))?;
    let upem = face.units_per_em() as f32;
    let scale = font_size / upem;

    let mut result: Vec<GlyphPathData> = Vec::new();
    let mut cursor_x: f32 = 0.0;

    for ch in text.chars() {
        let gid = match face.glyph_index(ch) {
            Some(id) => id,
            None => {
                // Advance by the space width and skip
                if let Some(sid) = face.glyph_index(' ') {
                    let adv =
                        face.glyph_hor_advance(sid).unwrap_or((upem * 0.5) as u16) as f32;
                    cursor_x += adv * scale;
                } else {
                    cursor_x += font_size * 0.5;
                }
                continue;
            }
        };

        let adv = face.glyph_hor_advance(gid).unwrap_or(0) as f32;
        let mut builder = GlyphPathBuilder {
            cmds: Vec::new(),
            cursor_x,
            scale,
            cur_x: 0.0,
            cur_y: 0.0,
        };
        let has_outline = face.outline_glyph(gid, &mut builder).is_some();

        if has_outline && !builder.cmds.is_empty() {
            result.push(GlyphPathData {
                commands: builder.cmds,
                width: adv * scale,
                character: ch,
            });
        }

        cursor_x += adv * scale;
    }

    if result.is_empty() {
        return Err("no renderable glyphs found in text".to_string());
    }

    Ok(result)
}

// ─── ttf-parser OutlineBuilder adapter ───────────────────────────────────────
//
// `ttf-parser` calls these methods while walking the glyph outline.
// Glyph coordinates are in font design units (integer).  We scale them
// to world-space pixels here (dividing by upem and multiplying by font_size).
// The Y axis is already +up in TTF, which matches our 2D scene.

struct GlyphPathBuilder {
    cmds: Vec<PathCommand>,
    cursor_x: f32,
    scale: f32,
    cur_x: f32,
    cur_y: f32,
}

impl ttf_parser::OutlineBuilder for GlyphPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let nx = x * self.scale + self.cursor_x;
        let ny = y * self.scale;
        self.cmds.push(PathCommand::MoveTo(Vec2::new(nx, ny)));
        self.cur_x = nx;
        self.cur_y = ny;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let nx = x * self.scale + self.cursor_x;
        let ny = y * self.scale;
        self.cmds.push(PathCommand::LineTo(Vec2::new(nx, ny)));
        self.cur_x = nx;
        self.cur_y = ny;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Convert quadratic Bézier (P0, P1, P2) → cubic (P0, C1, C2, P3)
        //   C1 = P0 + 2/3*(P1-P0)
        //   C2 = P2 + 2/3*(P1-P2)
        let q0x = self.cur_x;
        let q0y = self.cur_y;
        let q1x = x1 * self.scale + self.cursor_x;
        let q1y = y1 * self.scale;
        let q2x = x  * self.scale + self.cursor_x;
        let q2y = y  * self.scale;

        let c1x = q0x + (2.0 / 3.0) * (q1x - q0x);
        let c1y = q0y + (2.0 / 3.0) * (q1y - q0y);
        let c2x = q2x + (2.0 / 3.0) * (q1x - q2x);
        let c2y = q2y + (2.0 / 3.0) * (q1y - q2y);

        self.cmds.push(PathCommand::CubicTo(
            Vec2::new(c1x, c1y),
            Vec2::new(c2x, c2y),
            Vec2::new(q2x, q2y),
        ));
        self.cur_x = q2x;
        self.cur_y = q2y;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let c1x = x1 * self.scale + self.cursor_x;
        let c1y = y1 * self.scale;
        let c2x = x2 * self.scale + self.cursor_x;
        let c2y = y2 * self.scale;
        let nx  = x  * self.scale + self.cursor_x;
        let ny  = y  * self.scale;
        self.cmds.push(PathCommand::CubicTo(
            Vec2::new(c1x, c1y),
            Vec2::new(c2x, c2y),
            Vec2::new(nx, ny),
        ));
        self.cur_x = nx;
        self.cur_y = ny;
    }

    fn close(&mut self) {
        self.cmds.push(PathCommand::Close);
    }
}
