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
        Stmt::ForEach { var, list, body, span } => {
            let list_val = eval_expr(list, source, env)?;
            match list_val {
                Value::List(items) => {
                    let mut last = Value::Number(0.0);
                    for item in items {
                        env.define(var.clone(), item);
                        for stmt in &body {
                            last = eval_stmt(stmt.clone(), source, env)?;
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
}
