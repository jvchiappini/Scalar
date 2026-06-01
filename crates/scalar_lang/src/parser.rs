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
        
        // Unary minus
        let unary = just(Token::Minus).repeated()
            .then(call.or(atom))
            .foldr(|_op, target| {
                let span = target.span();
                Expr::UnaryMinus(Box::new(target), span)
            });

        // Binary operators with precedence: multiplicative (*, /) then additive (+, -)
        let mul_op = just(Token::Star).to(crate::ast::BinaryOp::Mul)
            .or(just(Token::Slash).to(crate::ast::BinaryOp::Div));

        let add_op = just(Token::Plus).to(crate::ast::BinaryOp::Add)
            .or(just(Token::Minus).to(crate::ast::BinaryOp::Sub));

        let multiplicative = unary.clone()
            .then(mul_op.then(unary).repeated())
            .foldl(|left, (op, right)| {
                let span = left.span().start..right.span().end;
                Expr::Binary { left: Box::new(left), op, right: Box::new(right), span }
            });

        let additive = multiplicative.clone()
            .then(add_op.then(multiplicative).repeated())
            .foldl(|left, (op, right)| {
                let span = left.span().start..right.span().end;
                Expr::Binary { left: Box::new(left), op, right: Box::new(right), span }
            });

        // Comparison operators (<, <=, >, >=, ==, !=) — lowest precedence
        let cmp_op = just(Token::Lt).to(crate::ast::BinaryOp::Lt)
            .or(just(Token::Le).to(crate::ast::BinaryOp::Le))
            .or(just(Token::Gt).to(crate::ast::BinaryOp::Gt))
            .or(just(Token::Ge).to(crate::ast::BinaryOp::Ge))
            .or(just(Token::Eq2).to(crate::ast::BinaryOp::Eq))
            .or(just(Token::Ne).to(crate::ast::BinaryOp::Ne));

        let comparison = additive.clone()
            .then(cmp_op.then(additive).repeated())
            .foldl(|left, (op, right)| {
                let span = left.span().start..right.span().end;
                Expr::Binary { left: Box::new(left), op, right: Box::new(right), span }
            });

        let base = comparison;

        // Method calls
        base.clone().then(
            just(Token::Dot)
                .ignore_then(select! { Token::Ident(i) => i })
                .then(args_and_kwargs)
                .repeated()
        )
        .foldl(|target, (method, (args, kwargs))| {
            let span = target.span().start..target.span().end;
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

        // Shared block parser: { stmt* }
        let block = stmt.clone()
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        // for var in expr .. expr { stmt* }       — range loop
        // for var in expr { stmt* }                — list iteration
        let for_stmt = just(Token::For)
            .ignore_then(select! { Token::Ident(i) => i })
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .then(
                // After the "in expr" part, decide: ".. end { body }" or "{ body }"
                just(Token::Dots)
                    .ignore_then(expr.clone())
                    .then(block.clone())
                    .map(|(end, body)| (Some(end), body))
                    .or(block.clone().map(|body| (None, body)))
            )
            .map_with_span(|((var, start), (end_opt, body)), span| {
                if let Some(end) = end_opt {
                    Stmt::For { var, start, end, body, span }
                } else {
                    Stmt::ForEach { var, list: start, body, span }
                }
            });

        let import_stmt = just(Token::Import)
            .ignore_then(select! { Token::String(s) => s })
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|path, span| Stmt::Import(path, span));

        // Assignment: ident = expr
        // Note: order matters — assign_stmt must be tried before expr_stmt
        // because both can start with an ident token.
        let assign_stmt = select! { Token::Ident(i) => i }
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|(name, value), span| Stmt::Assign { name, value, span });

        // Function definition: fn name(params) { body }
        let fn_def = just(Token::Fn)
            .ignore_then(select! { Token::Ident(i) => i })
            .then(
                select! { Token::Ident(p) => p }
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
            )
            .then(block.clone())
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|((name, params), body), span| Stmt::FnDef { name, params, body, span });

        // Conditional: if expr { stmt* } else { stmt* }
        let if_stmt = just(Token::If)
            .ignore_then(expr.clone())
            .then(block.clone())
            .then(
                just(Token::Else)
                    .ignore_then(block.clone())
                    .or_not()
            )
            .then_ignore(just(Token::Semi).or_not())
            .map_with_span(|((condition, then_body), else_body), span| {
                Stmt::If {
                    condition,
                    then_body,
                    else_body: else_body.unwrap_or_default(),
                    span,
                }
            });

        // Return: return expr?   (semicolons optional)
        let return_stmt = just(Token::Return)
            .then(
                expr.clone()
                    .then_ignore(just(Token::Semi).or_not())
                    .map(Some)
                    .or(just(Token::Semi).or_not().map(|_| None))
            )
            .map_with_span(|(_, value), span| Stmt::Return { value, span });

        let expr_stmt = expr.clone()
            .then_ignore(just(Token::Semi).or_not())
            .map(Stmt::Expr);

        let_stmt.or(for_stmt).or(import_stmt).or(fn_def)
            .or(if_stmt).or(return_stmt).or(assign_stmt).or(expr_stmt)
    });

    stmt.repeated()
        .then_ignore(end())
        .map(|statements| Ast { statements })
}
