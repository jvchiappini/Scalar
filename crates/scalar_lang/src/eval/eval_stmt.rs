use crate::ast::Stmt;
use crate::runtime::{Environment, Value};
use crate::eval::eval_expr::eval_expr;
use crate::eval::format_err;

/// Evaluates a statement.
pub fn eval_stmt(stmt: Stmt, source: &str, env: &mut Environment) -> Result<Value, String> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            let val = eval_expr(value, source, env)?;
            env.define(name, val.clone());
            Ok(val)
        },
        Stmt::For { var, start, end, body, span } => {
            let start_val = eval_expr(start, source, env)?;
            let end_val = eval_expr(end, source, env)?;
            
            if let (Value::Number(s), Value::Number(e)) = (start_val, end_val) {
                let mut last = Value::Number(0.0);
                for i in (s as i64)..(e as i64) {
                    env.define(var.clone(), Value::Number(i as f64));
                    for stmt in &body {
                        last = eval_stmt(stmt.clone(), source, env)?;
                    }
                }
                Ok(last)
            } else {
                Err(format_err(source, &span, "For loop range must be numbers"))
            }
        },
        Stmt::Expr(e) => eval_expr(e, source, env),
        Stmt::Import(path, span) => {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format_err(source, &span, format!("Failed to read import '{}': {}", path, e)))?;
            
            use logos::Logos;
            let tokens: Vec<_> = crate::lexer::Token::lexer(&content)
                .spanned()
                .filter_map(|(t, s)| t.ok().map(|token| (token, s)))
                .collect();

            let ast = crate::parser::parse(tokens)
                .map_err(|e| format!("Failed to parse import '{}': {:?}", path, e))?;

            let mut last_val = Value::Number(0.0);
            for s in ast.statements {
                last_val = eval_stmt(s, &content, env)?;
            }
            Ok(last_val)
        }
    }
}
