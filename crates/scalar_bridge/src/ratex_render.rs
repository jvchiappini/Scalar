//! # RaTeX — LaTeX Math Rendering Bridge
//!
//! Converts LaTeX expressions to vector path commands using the [RaTeX] crate
//! (pure-Rust KaTeX-compatible math layout engine).
//!
//! Pipeline:
//!   1. `ratex_parser::parse()` — LaTeX string → AST (`Vec<ParseNode>`)
//!   2. `ratex_layout::engine::layout()` — AST → layout tree (`LayoutBox`)
//!   3. `ratex_layout::to_display_list()` — layout tree → flat display list
//!   4. `display_list_to_paths()` — display list → `Vec<PathCommand>`
//!
//! [RaTeX]: https://github.com/erweixin/RaTeX

use std::collections::HashMap;

use ferrous_core::scene::world::types::PathCommand;
use ferrous_engine::glam::Vec2;
use ratex_layout::engine::layout;
use ratex_layout::to_display_list;
use ratex_layout::LayoutOptions;

/// Renders a LaTeX math expression to a list of path commands.
///
/// # Arguments
/// * `expr` — LaTeX math expression (e.g. `"E = mc^2"`, `"\\frac{a}{b}"`)
/// * `font_size` — Desired font size in pixels (applied as the base em)
///
/// # Returns
/// * `Ok((Vec<PathCommand>, f32))` — Path commands and the total width in pixels
/// * `Err(String)` — Human-readable error message
pub fn render_math(expr: &str, font_size: f32) -> Result<(Vec<PathCommand>, f32), String> {
    let nodes = parse_and_layout(expr, font_size)?;
    let (dl, width_px) = nodes;
    let cmds = display_list_to_paths(&dl, font_size)?;
    Ok((cmds, width_px))
}

/// Renders a LaTeX expression and returns **one group of path commands per
/// display item** (glyph, line, rect, or SVG path).
///
/// Use this to animate each piece individually (e.g., `FadeIn` each glyph
/// of a formula sequentially, like Manim's `Write`).
///
/// # Returns
/// * `Ok((Vec<Vec<PathCommand>>, f32))` — Per-item path groups and total width
/// * `Err(String)` — Human-readable error message
pub fn render_math_split(expr: &str, font_size: f32) -> Result<(Vec<Vec<PathCommand>>, f32), String> {
    let nodes = parse_and_layout(expr, font_size)?;
    let (dl, width_px) = nodes;
    let items = display_list_to_item_paths(&dl, font_size)?;
    Ok((items, width_px))
}

/// Shared parse + layout step used by both `render_math` and `render_math_split`.
fn parse_and_layout(expr: &str, font_size: f32) -> Result<(ratex_types::DisplayList, f32), String> {
    let nodes = ratex_parser::parse(expr).map_err(|e| format!("Parse error: {e}"))?;
    let opts = LayoutOptions::default();
    let root = layout(&nodes, &opts);
    let dl = to_display_list(&root);
    let width_px = dl.width as f32 * font_size;
    Ok((dl, width_px))
}

// ─── DisplayList → PathCommand conversion ─────────────────────────────────

/// Converts a RaTeX `DisplayList` (em units, y-down) to our `PathCommand`
/// format (pixels, y-up), applying `font_size` as the pixel scale.
///
/// Coordinate transform:
///   RaTeX: origin = top-left, y-down
///   Our:   y-up, baseline at the formula's anchor point
///   `our_x = ratex_x * font_size`
///   `our_y = (dl.height - ratex_y) * font_size`
fn display_list_to_paths(
    dl: &ratex_types::DisplayList,
    font_size: f32,
) -> Result<Vec<PathCommand>, String> {
    let dlh = dl.height as f32;
    let fs = font_size;
    let mut face_cache: HashMap<ratex_font::FontId, ttf_parser::Face<'static>> = HashMap::new();
    let mut cmds: Vec<PathCommand> = Vec::new();
    for item in &dl.items {
        convert_item(item, dlh, fs, &mut face_cache, &mut cmds)?;
    }
    Ok(cmds)
}

/// Converts a single RaTeX `DisplayItem` to path commands, using a shared
/// `face_cache` for font face reuse across items.
fn convert_item(
    item: &ratex_types::DisplayItem,
    dlh: f32,
    fs: f32,
    face_cache: &mut HashMap<ratex_font::FontId, ttf_parser::Face<'static>>,
    out: &mut Vec<PathCommand>,
) -> Result<(), String> {
    use ratex_types::DisplayItem;
    match item {
        DisplayItem::GlyphPath { x, y, scale, font, char_code, color: _color } => {
            let font_id = ratex_font::FontId::parse(font)
                .unwrap_or(ratex_font::FontId::MainRegular);
            let face = if !face_cache.contains_key(&font_id) {
                let ttf_name = format!("KaTeX_{font_id}.ttf");
                let font_data = ratex_katex_fonts::ttf_bytes(&ttf_name).ok_or_else(|| {
                    format!("RaTeX: embedded KaTeX font '{ttf_name}' not found")
                })?;
                let owned = font_data.to_vec();
                let leaked: &'static [u8] = Box::leak(owned.into_boxed_slice());
                let face = ttf_parser::Face::parse(leaked, 0)
                    .map_err(|e| format!("RaTeX: failed to parse KaTeX font '{ttf_name}': {e:?}"))?;
                face_cache.insert(font_id, face);
                face_cache.get(&font_id).unwrap()
            } else {
                face_cache.get(&font_id).unwrap()
            };
            let ch = char::from_u32(*char_code)
                .ok_or_else(|| format!("RaTeX: invalid Unicode U+{char_code:04X}"))?;
            let glyph_id = face.glyph_index(ch)
                .ok_or_else(|| format!("RaTeX: glyph U+{char_code:04X} not in font '{font_id}'"))?;
            let upem = face.units_per_em() as f32;
            let cursor_x = *x as f32 * fs;
            let cursor_y = (dlh - *y as f32) * fs;
            let glyph_scale = *scale as f32 * fs / upem;
            let mut builder = GlyphPathBuilder {
                cmds: Vec::new(),
                cursor_x,
                cursor_y,
                scale: glyph_scale,
                cur_x: 0.0,
                cur_y: 0.0,
            };
            face.outline_glyph(glyph_id, &mut builder);
            out.extend(builder.cmds);
        }
        DisplayItem::Line { x, y, width, thickness, color: _color, dashed: _dashed } => {
            let half_t = *thickness as f32 * fs / 2.0;
            let x0 = *x as f32 * fs;
            let x1 = (*x + width) as f32 * fs;
            let cy = (dlh - *y as f32) * fs;
            let top = cy + half_t;
            let bot = cy - half_t;
            out.push(PathCommand::MoveTo(Vec2::new(x0, top)));
            out.push(PathCommand::LineTo(Vec2::new(x1, top)));
            out.push(PathCommand::LineTo(Vec2::new(x1, bot)));
            out.push(PathCommand::LineTo(Vec2::new(x0, bot)));
            out.push(PathCommand::Close);
        }
        DisplayItem::Rect { x, y, width, height, color: _color } => {
            let x0 = *x as f32 * fs;
            let x1 = (*x + width) as f32 * fs;
            let y_top = (dlh - *y as f32) * fs;
            let y_bot = (dlh - (*y + height) as f32) * fs;
            out.push(PathCommand::MoveTo(Vec2::new(x0, y_top)));
            out.push(PathCommand::LineTo(Vec2::new(x1, y_top)));
            out.push(PathCommand::LineTo(Vec2::new(x1, y_bot)));
            out.push(PathCommand::LineTo(Vec2::new(x0, y_bot)));
            out.push(PathCommand::Close);
        }
        DisplayItem::Path { x, y, commands, fill: _fill, color: _color } => {
            let ox = *x as f32 * fs;
            let oy = (dlh - *y as f32) * fs;
            for cmd in commands {
                match cmd {
                    ratex_types::PathCommand::MoveTo { x: cx, y: cy } => {
                        out.push(PathCommand::MoveTo(Vec2::new(ox + *cx as f32 * fs, oy - *cy as f32 * fs)));
                    }
                    ratex_types::PathCommand::LineTo { x: cx, y: cy } => {
                        out.push(PathCommand::LineTo(Vec2::new(ox + *cx as f32 * fs, oy - *cy as f32 * fs)));
                    }
                    ratex_types::PathCommand::CubicTo { x1, y1, x2, y2, x, y } => {
                        out.push(PathCommand::CubicTo(
                            Vec2::new(ox + *x1 as f32 * fs, oy - *y1 as f32 * fs),
                            Vec2::new(ox + *x2 as f32 * fs, oy - *y2 as f32 * fs),
                            Vec2::new(ox + *x as f32 * fs, oy - *y as f32 * fs),
                        ));
                    }
                    ratex_types::PathCommand::QuadTo { x1, y1, x, y } => {
                        let (prev_x, prev_y) = last_point(out).unwrap_or((ox, oy));
                        let q1x = ox + *x1 as f32 * fs;
                        let q1y = oy - *y1 as f32 * fs;
                        let q2x = ox + *x as f32 * fs;
                        let q2y = oy - *y as f32 * fs;
                        let c1x = prev_x + (2.0 / 3.0) * (q1x - prev_x);
                        let c1y = prev_y + (2.0 / 3.0) * (q1y - prev_y);
                        let c2x = q2x + (2.0 / 3.0) * (q1x - q2x);
                        let c2y = q2y + (2.0 / 3.0) * (q1y - q2y);
                        out.push(PathCommand::CubicTo(
                            Vec2::new(c1x, c1y),
                            Vec2::new(c2x, c2y),
                            Vec2::new(q2x, q2y),
                        ));
                    }
                    ratex_types::PathCommand::Close => {
                        out.push(PathCommand::Close);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Like `display_list_to_paths`, but returns **one `Vec<PathCommand>` per
/// display item** instead of flattening everything into one list.
///
/// This enables per-glyph / per-piece animation of a formula.
fn display_list_to_item_paths(
    dl: &ratex_types::DisplayList,
    font_size: f32,
) -> Result<Vec<Vec<PathCommand>>, String> {
    let dlh = dl.height as f32;
    let fs = font_size;
    let mut face_cache: HashMap<ratex_font::FontId, ttf_parser::Face<'static>> = HashMap::new();
    let mut items: Vec<Vec<PathCommand>> = Vec::with_capacity(dl.items.len());
    for item in &dl.items {
        let mut cmds = Vec::new();
        convert_item(item, dlh, fs, &mut face_cache, &mut cmds)?;
        items.push(cmds);
    }
    Ok(items)
}

/// Returns the last (x, y) point from a sequence of `PathCommand`s.
///
/// This is used for converting quadratic Bézier curves (QuadTo) to cubic ones
/// (CubicTo), where the implicit start point is the previous command's endpoint.
fn last_point(cmds: &[PathCommand]) -> Option<(f32, f32)> {
    for cmd in cmds.iter().rev() {
        match cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => return Some((p.x, p.y)),
            PathCommand::CubicTo(_, _, p) => return Some((p.x, p.y)),
            PathCommand::Close => {} // skip Close, keep looking backward
        }
    }
    None
}

// ─── ttf-parser OutlineBuilder adapter ────────────────────────────────────
//
// Same pattern as in `imports.rs` — adapts `ttf_parser::OutlineBuilder`
// to emit `PathCommand`s, with coordinate transform applied.
//
// RaTeX glyph positioning:
//   cursor_x, cursor_y  — where to place the glyph (our y-up space)
//   scale               — glyph outline scale factor (includes upem conversion)

struct GlyphPathBuilder {
    cmds: Vec<PathCommand>,
    /// Absolute X position of this glyph's origin (our coords).
    cursor_x: f32,
    /// Absolute Y position of this glyph's origin (our coords).
    cursor_y: f32,
    /// Scale factor: font_size / units_per_em (per-glyph).
    scale: f32,
    /// Current pen X (relative to cursor_x after outline walk).
    cur_x: f32,
    /// Current pen Y (relative to cursor_y after outline walk).
    cur_y: f32,
}

impl ttf_parser::OutlineBuilder for GlyphPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let nx = x * self.scale + self.cursor_x;
        // TTF y-up matches our y-up (no negation needed within a glyph).
        let ny = y * self.scale + self.cursor_y;
        self.cmds.push(PathCommand::MoveTo(Vec2::new(nx, ny)));
        self.cur_x = nx;
        self.cur_y = ny;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let nx = x * self.scale + self.cursor_x;
        let ny = y * self.scale + self.cursor_y;
        self.cmds.push(PathCommand::LineTo(Vec2::new(nx, ny)));
        self.cur_x = nx;
        self.cur_y = ny;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Convert quadratic Bézier (Q0, Q1, Q2) → cubic (P0, C1, C2, P3)
        //   C1 = Q0 + 2/3 * (Q1 - Q0)
        //   C2 = Q2 + 2/3 * (Q1 - Q2)
        let q0x = self.cur_x;
        let q0y = self.cur_y;

        let q1x = x1 * self.scale + self.cursor_x;
        let q1y = y1 * self.scale + self.cursor_y;
        let q2x = x * self.scale + self.cursor_x;
        let q2y = y * self.scale + self.cursor_y;

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
        let c1y = y1 * self.scale + self.cursor_y;
        let c2x = x2 * self.scale + self.cursor_x;
        let c2y = y2 * self.scale + self.cursor_y;
        let nx = x * self.scale + self.cursor_x;
        let ny = y * self.scale + self.cursor_y;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let (cmds, width) = render_math("x", 48.0).expect("should render");
        assert!(!cmds.is_empty(), "should produce path commands");
        assert!(width > 0.0, "width should be positive");
    }

    #[test]
    fn test_fraction() {
        let (cmds, _) = render_math("\\frac{a}{b}", 48.0).expect("should render");
        assert!(!cmds.is_empty(), "fraction should produce path commands");
    }

    #[test]
    fn test_integral() {
        let (cmds, _) = render_math("\\int_{0}^{\\infty} e^{-x^2} dx", 48.0)
            .expect("should render integral");
        assert!(!cmds.is_empty(), "integral should produce path commands");
    }

    #[test]
    fn test_invalid_latex() {
        let result = render_math("\\undefinedcommand", 48.0);
        assert!(result.is_err(), "undefined command should error");
    }

    #[test]
    fn test_matrix_expression() {
        // This is EXACTLY the expression from test_text_anim.scl Section 16
        // (after Scalar DSL escape processing: \\ → \, \\\\ → \\)
        let expr = "\\begin{bmatrix} a & b \\\\ c & d \\end{bmatrix}^{-1} = \\frac{1}{ad-bc} \\begin{bmatrix} d & -b \\\\ -c & a \\end{bmatrix}";
        let (cmds, width) = render_math(expr, 48.0).expect("matrix should parse and render");
        assert!(!cmds.is_empty(), "matrix should produce path commands");
        assert!(width > 0.0, "width should be positive for matrix");
        eprintln!("Matrix: {} commands, width={}px", cmds.len(), width);
    }

    #[test]
    fn test_empty_expression() {
        let (cmds, width) = render_math("", 48.0).expect("empty should render");
        assert!(cmds.is_empty(), "empty should produce no paths");
        assert!((width - 0.0).abs() < 0.01, "empty width should be 0");
    }

    #[test]
    fn test_split_returns_per_item() {
        // `x` should produce at least 1 item (the 'x' glyph)
        let (items, _) = render_math_split("x", 48.0).expect("should render");
        assert!(!items.is_empty(), "should have at least one item");
        for (i, cmds) in items.iter().enumerate() {
            assert!(!cmds.is_empty(), "item {i} should have path commands");
        }
    }

    #[test]
    fn test_split_sum_matches_flat() {
        // The total path count across all items should equal the flat output
        let expr = "\\frac{a}{b}";
        let (flat_cmds, _) = render_math(expr, 48.0).expect("should render");
        let (split_items, _) = render_math_split(expr, 48.0).expect("should render");
        let split_count: usize = split_items.iter().map(|c| c.len()).sum();
        assert_eq!(flat_cmds.len(), split_count,
            "total path commands in split should equal flat total");
        // A fraction has at least 2 items: 'a' glyph, fraction bar, 'b' glyph
        assert!(split_items.len() >= 2, "fraction should split into ≥2 items, got {}", split_items.len());
        eprintln!("render_math_split: {} items, {} total commands", split_items.len(), split_count);
    }

    #[test]
    fn test_split_matrix() {
        let expr = "\\begin{bmatrix} a & b \\\\ c & d \\end{bmatrix}";
        let (items, _) = render_math_split(expr, 48.0).expect("matrix should render");
        assert!(items.len() >= 2, "matrix should split into ≥2 items, got {}", items.len());
        for (i, cmds) in items.iter().enumerate() {
            assert!(!cmds.is_empty(), "matrix item {i} should have paths");
        }
        eprintln!("Matrix split: {} items", items.len());
    }
}
