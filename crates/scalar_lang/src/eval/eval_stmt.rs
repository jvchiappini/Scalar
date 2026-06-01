use crate::ast::Stmt;
use crate::runtime::{Environment, Value};
use crate::eval::eval_expr::eval_expr;
use crate::eval::format_err;

/// Returns true if a value is "truthy" for `if` conditions.
pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        _ => true, // non-null/non-zero values are truthy
    }
}

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
                        if matches!(last, Value::Return(_)) {
                            return Ok(last);
                        }
                    }
                }
                Ok(last)
            } else {
                Err(format_err(source, &span, "For loop range must be numbers"))
            }
        },
        Stmt::ForEach { var, list, body, span } => {
            let list_val = eval_expr(list, source, env)?;
            match list_val {
                Value::List(items) => {
                    let mut last = Value::Number(0.0);
                    for item in items {
                        env.define(var.clone(), item);
                        for stmt in &body {
                            last = eval_stmt(stmt.clone(), source, env)?;
                            if matches!(last, Value::Return(_)) {
                                return Ok(last);
                            }
                        }
                    }
                    Ok(last)
                }
                _ => Err(format_err(source, &span, "ForEach loop requires a list value")),
            }
        },
        Stmt::Assign { name, value, span } => {
            let val = eval_expr(value, source, env)?;
            if env.get(&name).is_some() {
                env.define(name, val.clone());
                Ok(val)
            } else {
                Err(format_err(source, &span, format!("Cannot assign to undefined variable '{name}'")))
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
        Stmt::FnDef { name, params, body, .. } => {
            env.define(name, Value::Fn {
                params,
                body,
                source: source.to_string(),
            });
            Ok(Value::Number(0.0))
        }
        Stmt::If { condition, then_body, else_body, span: _ } => {
            let cond_val = eval_expr(condition, source, env)?;
            let branch = if is_truthy(&cond_val) { then_body } else { else_body };
            let mut last = Value::Number(0.0);
            for stmt in branch {
                last = eval_stmt(stmt, source, env)?;
                if matches!(last, Value::Return(_)) {
                    return Ok(last);
                }
            }
            Ok(last)
        }
        Stmt::Return { value, span: _ } => {
            let val = match value {
                Some(v) => eval_expr(v, source, env)?,
                None => Value::Number(0.0),
            };
            Ok(Value::Return(Box::new(val)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_script(script: &str) -> Result<Value, String> {
        use logos::Logos;
        let tokens: Vec<_> = crate::lexer::Token::lexer(script)
            .spanned()
            .filter_map(|(t, s)| t.ok().map(|token| (token, s)))
            .collect();
        let ast = crate::parser::parse(tokens)
            .map_err(|e| format!("Parse error: {:?}", e))?;
        let mut env = Environment::new();
        let mut last = Value::Number(0.0);
        for stmt in ast.statements {
            last = eval_stmt(stmt, script, &mut env)?;
        }
        Ok(last)
    }

    #[test]
    fn test_assign() {
        let val = eval_script("let x = 5; x = 10").unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 10.0).abs() < 1e-10));
    }

    #[test]
    fn test_binary_add() {
        let val = eval_script("let x = 3 + 4").unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 7.0).abs() < 1e-10));
    }

    #[test]
    fn test_binary_mixed() {
        let val = eval_script("let x = 2 + 3 * 4").unwrap();
        // Precedence: 2 + (3 * 4) = 14
        assert!(matches!(val, Value::Number(n) if (n - 14.0).abs() < 1e-10));
    }

    #[test]
    fn test_binary_div() {
        let val = eval_script("let x = 10 / 2").unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 5.0).abs() < 1e-10));
    }

    #[test]
    fn test_foreach_sum() {
        // Sum elements of a list using for-each + assign
        let val = eval_script(
            "let items = [1, 2, 3, 4, 5]; let sum = 0; for x in items { sum = sum + x }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 15.0).abs() < 1e-10));
    }

    #[test]
    fn test_foreach_empty() {
        let val = eval_script(
            "let items = []; let count = 0; for x in items { count = count + 1 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 0.0).abs() < 1e-10));
    }

    #[test]
    fn test_assign_undefined_error() {
        let result = eval_script("y = 5");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined"));
    }

    // ─── If / Return tests ───────────────────────────────────────────────────

    #[test]
    fn test_if_true() {
        let val = eval_script(
            "if 1 { 42 } else { 0 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 42.0).abs() < 1e-10));
    }

    #[test]
    fn test_if_false() {
        let val = eval_script(
            "if 0 { 42 } else { 99 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 99.0).abs() < 1e-10));
    }

    #[test]
    fn test_if_boolean() {
        let val = eval_script(
            "if true { 10 } else { 20 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 10.0).abs() < 1e-10));
    }

    #[test]
    fn test_if_no_else() {
        let val = eval_script(
            "let x = 5; if false { x = 99 }; x"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 5.0).abs() < 1e-10));
    }

    #[test]
    fn test_return_from_fn() {
        let val = eval_script(
            "fn half(x) { return x / 2; 999 }; half(10)"
        ).unwrap();
        // return should exit before 999
        assert!(matches!(val, Value::Number(n) if (n - 5.0).abs() < 1e-10));
    }

    #[test]
    fn test_return_immediate() {
        let val = eval_script(
            "fn early() { return 42 }; early()"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 42.0).abs() < 1e-10));
    }

    #[test]
    fn test_return_no_value() {
        let val = eval_script(
            "fn nothing() { return; 999 }; nothing()"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 0.0).abs() < 1e-10));
    }

    #[test]
    fn test_factorial_recursion() {
        // With if/else + comparison operators, we can write real recursion!
        let val = eval_script(
            "fn fact(n) { if n <= 1 { 1 } else { n * fact(n - 1) } }; fact(5)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 120.0).abs() < 1e-10));
    }

    #[test]
    fn test_fibonacci_recursion() {
        let val = eval_script(
            "fn fib(n) { if n <= 1 { n } else { fib(n - 1) + fib(n - 2) } }; fib(10)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 55.0).abs() < 1e-10));
    }

    #[test]
    fn test_return_in_loop() {
        let val = eval_script(
            "fn find(items, target) {
                for x in items {
                    if x == target { return x }
                }
                return 0
            }; find([1, 2, 3, 4, 5], 3)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 3.0).abs() < 1e-10));
    }

    #[test]
    fn test_chain_comparison() {
        // Single comparison operators work
        let val = eval_script(
            "if 5 > 3 { 100 } else { 200 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 100.0).abs() < 1e-10));
        // Multiple comparisons work via chained binary ops
        // Note: logical && is not implemented yet
        let val2 = eval_script(
            "if 10 >= 5 { 1 } else { 0 }"
        ).unwrap();
        assert!(matches!(val2, Value::Number(n) if (n - 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_eq_comparison() {
        let val = eval_script(
            "if 3 == 3 { 1 } else { 0 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_ne_comparison() {
        let val = eval_script(
            "if 3 != 4 { 1 } else { 0 }"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 1.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_def_and_call() {
        // Define a function with two params, call it
        let val = eval_script(
            "fn add(a, b) { a + b }; add(3, 4)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 7.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_no_args() {
        // Function with no args
        let val = eval_script(
            "fn answer() { 42 }; answer()"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 42.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_last_value() {
        // Function returns the last expression value
        let val = eval_script(
            "fn compute(x) { let y = x * 2; y + 1 }; compute(5)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 11.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_scope_isolation() {
        // Variables defined inside function must NOT leak to outer scope
        let val = eval_script(
            "fn foo() { let inside = 99 }; foo(); let x = 5"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 5.0).abs() < 1e-10));
        // Also verify 'inside' is not accessible in outer scope
        let result = eval_script(
            "fn foo() { let inside = 99 }; foo(); inside"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_fn_outer_access() {
        // Function can access variables from outer scope
        let val = eval_script(
            "let factor = 10; fn scale(x) { x * factor }; scale(5)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 50.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_param_shadows_outer() {
        // Parameter shadows outer variable with same name
        let val = eval_script(
            "let x = 100; fn add(x, y) { x + y }; add(3, 4)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 7.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_recursion() {
        // Recursive function via mutual looping (approximation without `if`)
        // We can test simple self-call at least
        let val = eval_script(
            "fn dec(n) { n - 1 }; dec(5)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 4.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_missing_param_defaults_zero() {
        // Missing args default to 0
        let val = eval_script(
            "fn add(a, b) { a + b }; add(5)"
        ).unwrap();
        assert!(matches!(val, Value::Number(n) if (n - 5.0).abs() < 1e-10));
    }

    #[test]
    fn test_fn_kwargs_error() {
        // User functions don't accept kwargs
        let result = eval_script(
            "fn foo(x) { x }; foo(1, y: 2)"
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("keyword"));
    }
}
