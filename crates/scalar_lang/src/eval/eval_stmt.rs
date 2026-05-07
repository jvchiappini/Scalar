use crate::ast::Stmt;
use crate::runtime::{Environment, Value};
use crate::eval::eval_expr::eval_expr;

/// Evaluates a statement.
pub fn eval_stmt(stmt: Stmt, env: &mut Environment) -> Result<Value, String> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            let val = eval_expr(value, env)?;
            env.define(name, val.clone());
            Ok(val)
        },
        Stmt::For { var, start, end, body, .. } => {
            let start_val = eval_expr(start, env)?;
            let end_val = eval_expr(end, env)?;
            
            if let (Value::Number(s), Value::Number(e)) = (start_val, end_val) {
                let mut last = Value::Number(0.0);
                for i in (s as i64)..(e as i64) {
                    env.define(var.clone(), Value::Number(i as f64));
                    for s in &body {
                        last = eval_stmt(s.clone(), env)?;
                    }
                }
                Ok(last)
            } else {
                Err("For loop range must be numbers".to_string())
            }
        },
        Stmt::Expr(e) => eval_expr(e, env),
    }
}
