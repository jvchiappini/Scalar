use crate::ast::{Expr, BinaryOp};
use crate::runtime::{Environment, Value};
use crate::eval::format_err;
use crate::eval::eval_stmt::eval_stmt;

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
                        BinaryOp::Lt => return Ok(Value::Boolean(a < b)),
                        BinaryOp::Le => return Ok(Value::Boolean(a <= b)),
                        BinaryOp::Gt => return Ok(Value::Boolean(a > b)),
                        BinaryOp::Ge => return Ok(Value::Boolean(a >= b)),
                        BinaryOp::Eq => return Ok(Value::Boolean(a == b)),
                        BinaryOp::Ne => return Ok(Value::Boolean(a != b)),
                    };
                    Ok(Value::Number(result))
                }
                (Value::Boolean(a), Value::Boolean(b)) => {
                    match op {
                        BinaryOp::Eq => Ok(Value::Boolean(a == b)),
                        BinaryOp::Ne => Ok(Value::Boolean(a != b)),
                        _ => Err(format_err(source, &span, "Comparison operator not supported for booleans")),
                    }
                }
                _ => {
                    // Eq/Ne can work on any pair of same-typed values
                    match op {
                        BinaryOp::Eq => Ok(Value::Boolean(false)),
                        BinaryOp::Ne => Ok(Value::Boolean(true)),
                        _ => Err(format_err(source, &span, "Binary operators require numeric operands")),
                    }
                }
            }
        },
        Expr::Call { func, args, kwargs, span } => {
            let function = env.get(&func).ok_or_else(|| {
                format_err(source, &span, format!("Undefined function: `{}`", func))
            })?;
            match function {
                Value::NativeFunction(f) => {
                    let mut arg_values = Vec::new();
                    for a in args {
                        arg_values.push(eval_expr(a, source, env)?);
                    }
                    let mut kwarg_values = std::collections::HashMap::new();
                    for (k, v) in kwargs {
                        kwarg_values.insert(k, eval_expr(v, source, env)?);
                    }
                    f(arg_values, kwarg_values)
                }
                Value::Fn { params, body, source: fn_source } => {
                    if !kwargs.is_empty() {
                        return Err(format_err(source, &span,
                            format!("User-defined function `{}` does not accept keyword arguments", func)));
                    }
                    // Evaluate arguments in the current environment
                    let mut arg_values = Vec::new();
                    for a in args {
                        arg_values.push(eval_expr(a, source, env)?);
                    }
                    // Create child scope and bind parameters
                    let mut fn_env = env.fresh_child();
                    for (i, param) in params.iter().enumerate() {
                        let val = arg_values.get(i).cloned().unwrap_or(Value::Number(0.0));
                        fn_env.define(param.clone(), val);
                    }
                    // Evaluate the function body
                    let mut last = Value::Number(0.0);
                    for stmt in body {
                        last = eval_stmt(stmt, &fn_source, &mut fn_env)?;
                        // Unwrap Return sentinel to get the actual return value
                        if let Value::Return(val) = &last {
                            return Ok(*val.clone());
                        }
                    }
                    Ok(last)
                }
                _ => Err(format_err(source, &span, format!("`{}` is not a function", func)))
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
