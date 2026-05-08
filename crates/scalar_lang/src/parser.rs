use chumsky::prelude::*;
use crate::lexer::Token;
use crate::ast::{Ast, Expr, Stmt, Span};

/// Parsea a Scalar script into an AST.
pub fn parse(tokens: Vec<(Token, Span)>) -> Result<Ast, Vec<Simple<Token>>> {
    let len = tokens.last().map(|(_, s)| s.end).unwrap_or(0);
    let stream = chumsky::Stream::from_iter(len..len, tokens.into_iter());
    parser().parse(stream)
}

fn parser() -> impl Parser<Token, Ast, Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let val = select! {
            Token::Number(n) => f64::from_bits(n),
        }
        .map_with_span(|n, span| Expr::Number(n, span))
        .or(select! {
            Token::Ident(i) => i,
        }
        .map_with_span(|i, span| Expr::Ident(i, span)))
        .or(select! {
            Token::String(s) => s,
        }
        .map_with_span(|s, span| Expr::String(s, span)));

        let list = val.clone()
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map_with_span(|l, span| Expr::List(l, span));

        let atom = val.or(list);

        let call = select! { Token::Ident(i) => i }
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
            )
            .map_with_span(|(name, args), span| Expr::Call { func: name, args, span });

        let base = call.or(atom);

        // Method calls
        base.clone().then(
            just(Token::Dot)
                .ignore_then(select! { Token::Ident(i) => i })
                .then(
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                )
                .repeated()
        )
        .foldl(|target, (method, args)| {
            let span = target.span().start..args.last().map(|a| a.span().end).unwrap_or(target.span().end); // Approximate span
            Expr::MethodCall {
                target: Box::new(target),
                method,
                args,
                span,
            }
        })
    });

    let stmt = recursive(|stmt| {
        let let_stmt = just(Token::Let)
            .ignore_then(select! { Token::Ident(i) => i })
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|(name, value), span| Stmt::Let { name, value, span });

        let for_stmt = just(Token::For)
            .ignore_then(select! { Token::Ident(i) => i })
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .then_ignore(just(Token::Dots))
            .then(expr.clone())
            .then(
                stmt.clone()
                    .repeated()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
            )
            .map_with_span(|(((var, start), end), body), span| Stmt::For { var, start, end, body, span });

        let import_stmt = just(Token::Import)
            .ignore_then(select! { Token::String(s) => s })
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|path, span| Stmt::Import(path, span));

        let expr_stmt = expr.clone()
            .then_ignore(just(Token::Semi).or_not())
            .map(Stmt::Expr);

        let_stmt.or(for_stmt).or(import_stmt).or(expr_stmt)
    });

    stmt.repeated()
        .then_ignore(end())
        .map(|statements| Ast { statements })
}
