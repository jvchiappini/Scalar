use scalar_lang::lexer::Token;
use logos::Logos;

fn main() {
    let script = "Line(-300, 0, 300, 0, 20, 1.0, 0.0, 0.0)";
    let mut lex = Token::lexer(script);
    for (t, s) in lex.spanned() {
        println!("{:?} {:?}", t, s);
    }
}
