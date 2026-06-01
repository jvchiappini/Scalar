use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("import")]
    Import,
    #[token("fn")]
    Fn,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        // Process escape sequences: \\ → \, \" → ", \n → newline, etc.
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(ch);
            }
        }
        Some(result)
    })]
    String(String),

    #[regex(r"[0-9]*\.?[0-9]+", |lex| lex.slice().parse::<f64>().ok().map(|f| f.to_bits()))]
    Number(u64),

    #[token("..")]
    Dots,

    // Comparison operators (multi-char must come before single-char)
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("==")]
    Eq2,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,

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
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,

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

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_lex() {
        let script = "Resolution(1920, 1080)\nBackground(0.05, 0.05, 0.1)\nLine(-300, 0, 300, 0, 20, 1.0, 0.0, 0.0)";
        let mut lex = Token::lexer(script);
        for (t, s) in lex.spanned() {
            println!("{:?} {:?}", t, s);
        }
        panic!("Show output");
    }
}
