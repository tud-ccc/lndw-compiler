use crate::types::*;
use chumsky::prelude::*;

pub fn run_parser(input: &str) -> Result<Expr, LpErr> {
    parse_expr()
        .parse(input)
        .into_result()
        .map_err(|parse_errs| {
            LpErr::Parse(
                parse_errs
                    .first()
                    .map(|e| e.to_string())
                    .unwrap_or("[unknown error]".into()),
            )
        })
}

fn parse_expr<'a>() -> impl Parser<'a, &'a str, Expr> {
    recursive(|expr| {
        let ident = text::ascii::ident().padded();

        let int = text::int(10)
            .map(|s: &str| s.parse().unwrap())
            .map(Expr::Num);

        // a single atom, either an integer, a parenthesized expression or an identifier
        let atom = int
            .or(expr.delimited_by(just('('), just(')')))
            .or(ident.map(String::from).map(Expr::Var))
            .padded();

        // operations, both unary and binary
        let mul_op = one_of("*/").map(Operator::try_from).map(Result::unwrap);
        let add_op = one_of("+-").map(Operator::try_from).map(Result::unwrap);

        // ====== THE ACTUAL PARSER =====
        // we define parsers for operations based on precedence
        // First, unary expressions, which may occur 0..N times
        // Second, multiplications,
        // Third, additions.
        //
        // Each of the three steps repeatedly looks for the pattern and then moves on.
        // For example, in addition we look for the pattern: 1 unary expression,
        // followed by an arbitrary number of tuples of (op, unary_op), like so:
        //
        // expr = unary + (op + unary)*

        let unary = just('-')
            .padded()
            .repeated()
            .foldr(atom, |_op, rhs| Expr::UnaryOp(Operator::Sub, Box::new(rhs)));

        let product = unary
            .clone()
            .foldl(mul_op.then(unary).repeated(), |lhs, (op, rhs)| {
                Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
            });

        

        product
            .clone()
            .foldl(add_op.then(product).repeated(), |lhs, (op, rhs)| {
                Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
            })
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple_expr() -> Result<(), LpErr> {
        let expr = run_parser("(1 + 2)")?;

        assert_eq!(
            expr,
            Expr::BinaryOp(
                Box::new(Expr::Num(1)),
                Operator::Add,
                Box::new(Expr::Num(2))
            )
        );
        Ok(())
    }

    #[test]
    fn parse_simple_sym() -> Result<(), LpErr> {
        let expr = run_parser("(1 + a)")?;

        assert_eq!(
            expr,
            Expr::BinaryOp(
                Box::new(Expr::Num(1)),
                Operator::Add,
                Box::new(Expr::Var("a".to_string()))
            )
        );
        Ok(())
    }

    #[test]
    fn parse_nested_parens() -> Result<(), LpErr> {
        let expr = run_parser("((((1 + 2))))")?;

        assert_eq!(
            expr,
            Expr::BinaryOp(
                Box::new(Expr::Num(1)),
                Operator::Add,
                Box::new(Expr::Num(2))
            )
        );
        Ok(())
    }

    #[test]
    fn parse_invalid_expr() -> Result<(), LpErr> {
        let inputs = vec![
            "(1 + 2+)",
            "(1 + +2)",
            "(1 + *)",
            "(1 1 1)",
            "(1 (1) 1)",
            "(1 + 1 1)",
        ];

        for input in inputs {
            assert!(run_parser(input).is_err(), "`{input}` should fail but got `{:?}`", run_parser(input));
        }
        Ok(())
    }

    #[test]
    fn parse_nested_1() -> Result<(), LpErr> {
        let expr = run_parser("(1 + (2 * 3))")?;

        assert_eq!(
            expr,
            Expr::BinaryOp(
                Box::new(Expr::Num(1)),
                Operator::Add,
                Box::new(Expr::BinaryOp(
                    Box::new(Expr::Num(2)),
                    Operator::Mul,
                    Box::new(Expr::Num(3))
                ))
            )
        );
        Ok(())
    }

    #[test]
    fn parse_nested_2() -> Result<(), LpErr> {
        let expr = run_parser("((1 + 2) * 3)")?;

        assert_eq!(
            expr,
            Expr::BinaryOp(
                Box::new(Expr::BinaryOp(
                    Box::new(Expr::Num(1)),
                    Operator::Add,
                    Box::new(Expr::Num(2))
                )),
                Operator::Mul,
                Box::new(Expr::Num(3))
            )
        );
        Ok(())
    }
}
