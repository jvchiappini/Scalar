use crate::ast::{Expr, BinaryOp};
use crate::runtime::{Environment, Value};
use crate::eval::format_err;

/// Evaluates an expression.
pub fn eval_expr(expr: Expr, source: &str, env: &mut Environment) -> Result<Value, String> {
    match expr {
        Expr::Number(n, _) => Ok(Value::Number(n)),
        Expr::Ident(name, span) => env.get(&name).ok_or_else(|| {
            format_err(source, &span, format!("Undefined variable: `{}`", name))
        }),
        Expr::String(s, _) => Ok(Value::String(s)),
        Expr::UnaryMinus(target, span) => {
            let val = eval_expr(*target, source, env)?;
            if let Value::Number(n) = val {
                Ok(Value::Number(-n))
            } else {
                Err(format_err(source, &span, "Unary minus can only be applied to numbers"))
            }
        },
        Expr::List(exprs, _) => {
            let mut values = Vec::new();
            for e in exprs {
                values.push(eval_expr(e, source, env)?);
            }
            Ok(Value::List(values))
        },
        Expr::Binary { left, op, right, span } => {
            let l = eval_expr(*left, source, env)?;
            let r = eval_expr(*right, source, env)?;
            match (l, r) {
                (Value::Number(a), Value::Number(b)) => {
                    let result = match op {
                        BinaryOp::Add => a + b,
                        BinaryOp::Sub => a - b,
                        BinaryOp::Mul => a * b,
                        BinaryOp::Div => {
                            if b == 0.0 {
                                return Err(format_err(source, &span, "Division by zero"));
                            }
                            a / b
                        }
                    };
                    Ok(Value::Number(result))
                }
                _ => Err(format_err(source, &span, "Binary operators require numeric operands")),
            }
        },
        Expr::Call { func, args, kwargs, span } => {
            let function = env.get(&func).ok_or_else(|| {
                format_err(source, &span, format!("Undefined function: `{}`", func))
            })?;
            if let Value::NativeFunction(f) = function {
                let mut arg_values = Vec::new();
                for a in args {
                    arg_values.push(eval_expr(a, source, env)?);
                }
                let mut kwarg_values = std::collections::HashMap::new();
                for (k, v) in kwargs {
                    kwarg_values.insert(k, eval_expr(v, source, env)?);
                }
                f(arg_values, kwarg_values)
            } else {
                Err(format_err(source, &span, format!("`{}` is not a function", func)))
            }
        },
        Expr::MethodCall { target, method, args, kwargs, span } => {
            let target_val = eval_expr(*target, source, env)?;
            
            // Try to find method in object first
            let function = if let Value::Object(ref o) = target_val {
                o.get(&method).cloned()
            } else {
                None
            };
            
            let function = function.or_else(|| env.get(&method))
                .ok_or_else(|| {
                    format_err(source, &span, format!("Undefined method: `{}`", method))
                })?;
            
            if let Value::NativeFunction(f) = function {
                let mut arg_values = vec![target_val];
                for a in args {
                    arg_values.push(eval_expr(a, source, env)?);
                }
                let mut kwarg_values = std::collections::HashMap::new();
                for (k, v) in kwargs {
                    kwarg_values.insert(k, eval_expr(v, source, env)?);
                }
                f(arg_values, kwarg_values)
            } else {
                Err(format_err(source, &span, format!("`{}` is not a method", method)))
            }
        }
    }
}
