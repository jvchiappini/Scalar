use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    #[token("let")]
    Let,
    #[token("for")]
    For,
    #[token("in")]
    In,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    #[regex(r"[0-9]*\.?[0-9]+", |lex| lex.slice().parse::<f64>().ok().map(|f| f.to_bits()))]
    Number(u64),

    #[token("..")]
    Dots,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,
    #[token("=")]
    Eq,
    #[token(".")]
    Dot,
    #[token(";")]
    Semi,

    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

/// A Lexer for the Scalar language.
/// 
/// Uses `logos` to efficiently tokenize input strings into `Token` variants.
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}
