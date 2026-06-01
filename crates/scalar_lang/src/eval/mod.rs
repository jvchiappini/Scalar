pub mod eval_expr;
pub mod eval_stmt;

use crate::ast::{Ast, Span};
use crate::runtime::{Environment, Value};

/// Converts a byte offset in source text to 1-based (line, column).
///
/// Line numbers start at 1, column numbers start at 1.
pub fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let line = source[..offset].lines().count();
    let col = offset
        - source[..offset]
            .rfind('\n')
            .map(|i| i + 1) // skip past the newline
            .unwrap_or(0);
    (line, col + 1) // 1-indexed
}

/// Formats an error message with source location from a span.
pub fn format_err(source: &str, span: &Span, msg: impl std::fmt::Display) -> String {
    let (line, col) = offset_to_line_col(source, span.start);
    format!("{} (line {}, col {})", msg, line, col)
}

/// Evaluates a Scalar AST in the given environment.
///
/// `source` is the original script text, used for precise line:column error reporting.
pub fn evaluate(source: &str, ast: Ast, env: &mut Environment) -> Result<Value, String> {
    let mut last_value = Value::Number(0.0);
    for stmt in ast.statements {
        last_value = eval_stmt::eval_stmt(stmt, source, env)?;
    }
    Ok(last_value)
}
