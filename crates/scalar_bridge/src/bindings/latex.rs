//! # LaTeX Math Rendering for Scalar (RaTeX)
//!
//! Renders LaTeX math expressions as vector paths using [RaTeX], a pure-Rust
//! KaTeX-compatible math layout engine. No external tools are required — the
//! 19 KaTeX TrueType fonts are embedded at compile time via `ratex-katex-fonts`.
//!
//! Exposed functions:
//!
//! | Function | Description |
//! |----------|-------------|
//! | `Tex(expr, x, y, ...kwargs)` | Render a LaTeX math expression as vector paths |
//!
//! ## Tex Kwarg Reference
//!
//! | Kwarg | Type | Default | Description |
//! |-------|------|---------|-------------|
//! | `size` | Number | `48` | Font size in pixels |
//! | `x` | Number | `0` | X position |
//! | `y` | Number | `0` | Y position |
//! | `fill` | [r,g,b,a] | `[1,1,1,1]` | Fill color |
//! | `stroke` | [r,g,b,a] | — | Stroke color (no stroke if omitted) |
//! | `stroke_width` | Number | `2.0` | Stroke thickness |
//! | `opacity` | Number | `1.0` | Global opacity |
//! | `rotation` | Number | `0` | Rotation in degrees |
//! | `z_index` | Number | `0` | Z-order |
//!
//! ## Supported LaTeX constructs
//!
//! RaTeX supports ~99.5% of KaTeX syntax, including:
//!
//! - Fractions `\frac{a}{b}`
//! - Superscript `x^2` and subscript `a_i` with `{…}` grouping
//! - Square roots `\sqrt{x}`, `\sqrt[n]{x}`
//! - Large operators: `\sum`, `\int`, `\prod` with limits
//! - Greek letters: `\alpha`, `\beta`, `\pi`, `\sigma`, … 
//! - Math symbols: `\infty`, `\to`, `\partial`, `\times`, `\pm`, `\neq`, …
//! - Brackets: `\left(`, `\right)`, `\left[`, `\right]`
//! - Text: `\text{inline text}`
//! - Matrices: `\begin{matrix}…\end{matrix}`, `\begin{pmatrix}`, etc.
//! - Aligned equations: `\begin{aligned}…\end{aligned}`
//! - Accents: `\hat{x}`, `\tilde{x}`, `\bar{x}`, `\vec{x}`, …
//! - Font commands: `\mathbf`, `\mathcal`, `\mathfrak`, `\mathbb`, `\mathrm`, `\mathit`
//! - Colors: `\color{red}{x}`, `\colorbox{yellow}{text}`
//! - Chemistry: `\ce{H2SO4}`, `\pu{1.5e-3 mol//L}` (via mhchem)
//! - And many more (see [RaTeX docs](https://github.com/erweixin/RaTeX))
//!
//! [RaTeX]: https://github.com/erweixin/RaTeX

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use ferrous_engine::Renderer;
use scalar_lang::{Environment, Value};

use crate::bindings::shapes;
use crate::ratex_render;

/// Registers the `Tex()` function into the Scalar environment.
pub fn register(
    env: &mut Environment,
    renderer: Rc<RefCell<Renderer>>,
    _fonts: Rc<RefCell<Vec<crate::bindings::imports::FontEntry>>>,
) {
    let ren = renderer.clone();
    env.define(
        "Tex".to_string(),
        Value::NativeFunction(Rc::new(move |args, kwargs: HashMap<String, Value>| {
            // ── Read arguments ──────────────────────────────────────────────
            let expr = match args.get(0) {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(
                        "Tex(expr, x, y, ...): first argument must be a string \
                         (LaTeX math expression)"
                            .to_string(),
                    );
                }
            };

            let x = shapes::kwarg_num(&kwargs, "x", 0.0) as f32;
            let y = shapes::kwarg_num(&kwargs, "y", 0.0) as f32;
            let font_size = shapes::kwarg_num(&kwargs, "size", 48.0) as f32;

            // ── Render math expression to per‑item path groups ─────────────
            let (items, _width) = ratex_render::render_math_split(&expr, font_size)
                .map_err(|e| format!("Tex: {}", e))?;

            // ── Spawn one shape per display item ──────────────────────────
            // Each item (glyph, line, rect, path) is already positioned
            // correctly within the formula — all share the same (x, y) anchor.
            let mut r = ren.borrow_mut();
            // Remove size, x, y from kwargs so they don't leak into shape creation
            let mut clean_kwargs = kwargs.clone();
            clean_kwargs.remove("size");
            clean_kwargs.remove("x");
            clean_kwargs.remove("y");
            let sk = shapes::parse_shape_kwargs(&clean_kwargs);

            let mut ids: Vec<Value> = Vec::with_capacity(items.len());
            for cmds in items {
                if cmds.is_empty() {
                    continue;
                }
                let id = shapes::spawn_2d_shape_with_kwargs(&mut r, x, y, cmds, &sk);
                ids.push(Value::NodeId(id));
            }

            Ok(Value::List(ids))
        })),
    );
}
