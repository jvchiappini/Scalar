//! Mini math expression parser & evaluator for `Plot()`.
//!
//! Supports:
//! - Numbers: `3.14`, `-2.5`
//! - Variable: `x`
//! - Constants: `pi`, `e`
//! - Binary ops: `+`, `-`, `*`, `/`, `^` (right-assoc power)
//! - Unary minus
//! - Functions: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`,
//!   `sqrt`, `abs`, `log`, `ln`, `exp`, `floor`, `ceil`, `round`, `sign`
//! - Parentheses for grouping

use std::f64::consts;

// ── Tokens ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
    Eof,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '^' => { tokens.push(Token::Caret); chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '0'..='9' | '.' => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        num.push(d);
                        chars.next();
                    } else if d == 'e' || d == 'E' {
                        // scientific notation: 1.5e-3
                        num.push(d);
                        chars.next();
                        if let Some(&s) = chars.peek() {
                            if s == '+' || s == '-' {
                                num.push(s);
                                chars.next();
                            }
                        }
                    } else {
                        break;
                    }
                }
                let n: f64 = num.parse().map_err(|_| format!("Invalid number: {}", num))?;
                tokens.push(Token::Number(n));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            _ => return Err(format!("Unexpected character: '{}'", ch)),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

// ── AST ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Variable,
    UnaryMinus(Box<Expr>),
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

// ── Parser (Pratt) ──────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let got = self.advance();
        if std::mem::discriminant(&got) != std::mem::discriminant(expected) {
            return Err(format!("Expected {:?}, got {:?}", expected, got));
        }
        // Special case for Number/Ident which carry values
        match (expected, &got) {
            (Token::Number(_), Token::Number(_)) => Ok(()),
            (Token::Ident(_), Token::Ident(_)) => Ok(()),
            _ if got == *expected => Ok(()),
            _ => Err(format!("Expected {:?}, got {:?}", expected, got)),
        }
    }

    /// Precedence table (lowest first):
    ///   +, -     → 1
    ///   *, /     → 2
    ///   ^        → 3 (right-assoc)
    fn prefix_bp(&self, op: &Token) -> Result<u8, String> {
        match op {
            Token::Plus => Ok(3),
            Token::Minus => Ok(3),
            _ => Err(format!("Not a prefix operator: {:?}", op)),
        }
    }

    fn infix_bp(&self, op: &Token) -> Option<(u8, bool)> {
        match op {
            Token::Plus => Some((1, false)),
            Token::Minus => Some((1, false)),
            Token::Star => Some((2, false)),
            Token::Slash => Some((2, false)),
            Token::Caret => Some((3, true)),  // right-assoc
            _ => None,
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_atom()?;

        loop {
            let op = self.peek().clone();
            if op == Token::Eof || op == Token::RParen || op == Token::Comma {
                break;
            }

            if let Some((bp, left_assoc)) = self.infix_bp(&op) {
                if bp < min_bp {
                    break;
                }
                self.advance();
                let rhs_min = if left_assoc { bp } else { bp };
                let rhs = self.parse_expr(rhs_min)?;
                lhs = Expr::BinaryOp {
                    op: match op {
                        Token::Plus => BinaryOp::Add,
                        Token::Minus => BinaryOp::Sub,
                        Token::Star => BinaryOp::Mul,
                        Token::Slash => BinaryOp::Div,
                        Token::Caret => BinaryOp::Pow,
                        _ => unreachable!(),
                    },
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        let tok = self.advance();
        match tok {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Ident(name) => {
                // Check if it's a function call
                if self.peek() == &Token::LParen {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if self.peek() != &Token::RParen {
                        args.push(self.parse_expr(0)?);
                        while self.peek() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expr(0)?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call { func: name, args })
                } else {
                    match name.as_str() {
                        "x" | "X" => Ok(Expr::Variable),
                        "pi" => Ok(Expr::Number(consts::PI)),
                        "e" => Ok(Expr::Number(consts::E)),
                        _ => Err(format!("Unknown identifier: '{}'", name)),
                    }
                }
            }
            Token::Plus => {
                // unary plus is a no-op
                self.parse_expr(self.prefix_bp(&Token::Plus)?)
            }
            Token::Minus => {
                let rhs = self.parse_expr(self.prefix_bp(&Token::Minus)?)?;
                Ok(Expr::UnaryMinus(Box::new(rhs)))
            }
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", tok)),
        }
    }
}

// ── Evaluator ───────────────────────────────────────────────────────────────

fn eval(expr: &Expr, x: f64) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Variable => x,
        Expr::UnaryMinus(inner) => -eval(inner, x),
        Expr::BinaryOp { op, lhs, rhs } => {
            let l = eval(lhs, x);
            let r = eval(rhs, x);
            match op {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
                BinaryOp::Div => l / r,
                BinaryOp::Pow => l.powf(r),
            }
        }
        Expr::Call { func, args } => {
            let evaluated: Vec<f64> = args.iter().map(|a| eval(a, x)).collect();
            match func.as_str() {
                "sin" => evaluated[0].sin(),
                "cos" => evaluated[0].cos(),
                "tan" => evaluated[0].tan(),
                "asin" => evaluated[0].asin(),
                "acos" => evaluated[0].acos(),
                "atan" => evaluated[0].atan(),
                "sqrt" => evaluated[0].sqrt(),
                "abs" => evaluated[0].abs(),
                "log" => evaluated[0].log10(),
                "ln" => evaluated[0].ln(),
                "exp" => evaluated[0].exp(),
                "floor" => evaluated[0].floor(),
                "ceil" => evaluated[0].ceil(),
                "round" => evaluated[0].round(),
                "sign" => evaluated[0].signum(),
                "atan2" => evaluated[0].atan2(evaluated[1]),
                "min" => evaluated[0].min(evaluated[1]),
                "max" => evaluated[0].max(evaluated[1]),
                "clamp" => evaluated[0].clamp(evaluated[1], evaluated[2]),
                _ => 0.0,
            }
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Parses and evaluates a mathematical expression for a given `x`.
///
/// # Examples
///
/// ```rust,ignore
/// // evaluate("x^2 + 2*x - 3", 5.0) -> 32.0
/// // evaluate("sin(x)", 0.0) -> 0.0
/// ```
pub fn evaluate(expr_str: &str, x: f64) -> Result<f64, String> {
    let tokens = tokenize(expr_str)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr(0)?;
    Ok(eval(&expr, x))
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_approx_eq(evaluate("2+3", 0.0).unwrap(), 5.0);
        assert_approx_eq(evaluate("10-3", 0.0).unwrap(), 7.0);
        assert_approx_eq(evaluate("4*5", 0.0).unwrap(), 20.0);
        assert_approx_eq(evaluate("20/4", 0.0).unwrap(), 5.0);
    }

    #[test]
    fn test_power() {
        assert_approx_eq(evaluate("2^3", 0.0).unwrap(), 8.0);
        assert_approx_eq(evaluate("3^2^3", 0.0).unwrap(), 6561.0); // right-assoc: 3^(2^3) = 3^8
    }

    #[test]
    fn test_variable() {
        assert_approx_eq(evaluate("x^2", 5.0).unwrap(), 25.0);
        assert_approx_eq(evaluate("x^2 + 2*x - 3", 5.0).unwrap(), 32.0);
    }

    #[test]
    fn test_unary_minus() {
        assert_approx_eq(evaluate("-5", 0.0).unwrap(), -5.0);
        assert_approx_eq(evaluate("-x", 3.0).unwrap(), -3.0);
        assert_approx_eq(evaluate("x^2 + -x", 5.0).unwrap(), 20.0);
    }

    #[test]
    fn test_parentheses() {
        assert_approx_eq(evaluate("2*(3+4)", 0.0).unwrap(), 14.0);
        assert_approx_eq(evaluate("(x+1)*(x-1)", 5.0).unwrap(), 24.0);
    }

    #[test]
    fn test_functions() {
        assert_approx_eq(evaluate("sin(0)", 0.0).unwrap(), 0.0);
        assert_approx_eq(evaluate("cos(0)", 0.0).unwrap(), 1.0);
        assert_approx_eq(evaluate("sqrt(16)", 0.0).unwrap(), 4.0);
        assert_approx_eq(evaluate("abs(-5)", 0.0).unwrap(), 5.0);
        assert_approx_eq(evaluate("round(3.7)", 0.0).unwrap(), 4.0);
        assert_approx_eq(evaluate("floor(3.7)", 0.0).unwrap(), 3.0);
        assert_approx_eq(evaluate("ceil(3.1)", 0.0).unwrap(), 4.0);
    }

    #[test]
    fn test_constants() {
        assert_approx_eq(evaluate("pi", 0.0).unwrap(), consts::PI);
        assert_approx_eq(evaluate("e", 0.0).unwrap(), consts::E);
    }

    #[test]
    fn test_precedence() {
        // 2 + 3 * 4 = 2 + 12 = 14
        assert_approx_eq(evaluate("2+3*4", 0.0).unwrap(), 14.0);
        // (2+3)*4 = 20
        assert_approx_eq(evaluate("(2+3)*4", 0.0).unwrap(), 20.0);
        // -x^2 should be -(x^2) not (-x)^2
        assert_approx_eq(evaluate("-x^2", 3.0).unwrap(), -9.0);
    }

    #[test]
    fn test_complex() {
        // sin(x)^2 + cos(x)^2 should be ~1 for any x
        let val = evaluate("sin(x)^2 + cos(x)^2", 1.5).unwrap();
        assert_approx_eq(val, 1.0);
    }

    fn assert_approx_eq(a: f64, b: f64) {
        if (a - b).abs() > 1e-10 {
            panic!("{} != {}", a, b);
        }
    }
}
