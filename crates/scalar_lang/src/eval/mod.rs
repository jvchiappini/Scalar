pub mod eval_expr;
pub mod eval_stmt;

use crate::ast::Ast;
use crate::runtime::{Environment, Value};
use std::rc::Rc;

/// Evaluates a Scalar AST in the given environment.
pub fn evaluate(ast: Ast, env: &mut Environment) -> Result<Value, String> {
    let mut last_value = Value::Number(0.0);
    for stmt in ast.statements {
        last_value = eval_stmt::eval_stmt(stmt, env)?;
    }
    Ok(last_value)
}
