use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64, Span),
    Ident(String, Span),
    String(String, Span),
    UnaryMinus(Box<Expr>, Span),
    List(Vec<Expr>, Span),
    /// Binary operation: left op right
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    /// Method call: target.method(args)
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        kwargs: std::collections::HashMap<String, Expr>,
        span: Span,
    },
    /// Function call: func(args)
    Call {
        func: String,
        args: Vec<Expr>,
        kwargs: std::collections::HashMap<String, Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, s) => s.clone(),
            Expr::Ident(_, s) => s.clone(),
            Expr::String(_, s) => s.clone(),
            Expr::UnaryMinus(_, s) => s.clone(),
            Expr::List(_, s) => s.clone(),
            Expr::Binary { span, .. } => span.clone(),
            Expr::MethodCall { span, .. } => span.clone(),
            Expr::Call { span, .. } => span.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        span: Span,
    },
    For {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Expr(Expr),
    Import(String, Span),
    ForEach {
        var: String,
        list: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Assign {
        name: String,
        value: Expr,
        span: Span,
    },
}

/// Abstract Syntax Tree node for Scalar.
/// 
/// All nodes track their `Span` for error reporting and traceability.
#[derive(Debug, Clone)]
pub struct Ast {
    pub statements: Vec<Stmt>,
}
