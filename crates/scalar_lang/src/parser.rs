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

        let arg = select! { Token::Ident(i) => i }
            .then_ignore(just(Token::Colon))
            .then(expr.clone())
            .map(|(name, val)| (Some(name), val))
            .or(expr.clone().map(|val| (None, val)));

        let args_and_kwargs = arg
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .map(|list| {
                let mut positional = Vec::new();
                let mut named = std::collections::HashMap::new();
                for (name, val) in list {
                    if let Some(n) = name {
                        named.insert(n, val);
                    } else {
                        positional.push(val);
                    }
                }
                (positional, named)
            });

        let call = select! { Token::Ident(i) => i }
            .then(args_and_kwargs.clone())
            .map_with_span(|(name, (args, kwargs)), span| Expr::Call { func: name, args, kwargs, span });

        let atom = val.or(list);
        let base = call.or(atom);

        // Method calls
        base.clone().then(
            just(Token::Dot)
                .ignore_then(select! { Token::Ident(i) => i })
                .then(args_and_kwargs)
                .repeated()
        )
        .foldl(|target, (method, (args, kwargs))| {
            let span = target.span().start..target.span().end; // Optimized span
            Expr::MethodCall {
                target: Box::new(target),
                method,
                args,
                kwargs,
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
