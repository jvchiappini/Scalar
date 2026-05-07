use crate::ast::Expr;
use crate::runtime::{Environment, Value};

/// Evaluates an expression.
pub fn eval_expr(expr: Expr, env: &mut Environment) -> Result<Value, String> {
    match expr {
        Expr::Number(n, _) => Ok(Value::Number(n)),
        Expr::Ident(name, _) => env.get(&name).ok_or_else(|| format!("Undefined variable: {}", name)),
        Expr::String(s, _) => Ok(Value::String(s)),
        Expr::List(exprs, _) => {
            let mut values = Vec::new();
            for e in exprs {
                values.push(eval_expr(e, env)?);
            }
            Ok(Value::List(values))
        },
        Expr::Call { func, args, .. } => {
            let function = env.get(&func).ok_or_else(|| format!("Undefined function: {}", func))?;
            if let Value::NativeFunction(f) = function {
                let mut arg_values = Vec::new();
                for a in args {
                    arg_values.push(eval_expr(a, env)?);
                }
                f(arg_values)
            } else {
                Err(format!("'{}' is not a function", func))
            }
        },
        Expr::MethodCall { target, method, args, .. } => {
            let target_val = eval_expr(*target, env)?;
            let function = env.get(&method).ok_or_else(|| format!("Undefined method: {}", method))?;
            
            if let Value::NativeFunction(f) = function {
                let mut arg_values = vec![target_val];
                for a in args {
                    arg_values.push(eval_expr(a, env)?);
                }
                f(arg_values)
            } else {
                Err(format!("'{}' is not a method", method))
            }
        }
    }
}
