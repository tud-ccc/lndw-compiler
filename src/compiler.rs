#[derive(Debug)]
enum LpErr {
    // Tokenize(String),
    SExpr(String),
    Parse(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Token {
    /// (
    Lparen,
    /// )
    Rparen,
    Sym(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SExpr {
    Sym(String),
    List(Vec<SExpr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Expr {
    Num(i32),
    Var(String),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
}

fn tokenize(expr: &str) -> Result<Vec<Token>, LpErr> {
    expr.replace('(', "( ")
        .replace(')', " )")
        .split_ascii_whitespace()
        .map(|s| match s {
            "(" => Ok(Token::Lparen),
            ")" => Ok(Token::Rparen),
            _ => Ok(Token::Sym(s.to_string())),
            // "+" => Ok(Token::Op(Operator::Add)),
            // "-" => Ok(Token::Op(Operator::Sub)),
            // "*" => Ok(Token::Op(Operator::Mul)),
            // "/" => Ok(Token::Op(Operator::Div)),
            // _ => match s.parse::<i32>() {
            //     Ok(n) => Ok(Token::Num(n)),
            //     Err(_) => Ok(Token::Sym(s.to_string())),
            // }
            // _ => Err(LpErr::Tokenize(format!("invalid token `{s}`"))),
        })
        .collect::<Result<Vec<Token>, LpErr>>()
}

fn parse_sexpr(tokens: &mut Vec<Token>) -> Result<SExpr, LpErr> {
    match tokens.remove(0) {
        Token::Lparen => {
            let mut list = Vec::new();
            while !matches!(tokens.first(), Some(Token::Rparen)) {
                list.push(parse_sexpr(tokens)?);
                if tokens.is_empty() {
                    panic!("Unclosed list");
                }
            }
            assert_eq!(tokens.remove(0), Token::Rparen); // consume Rparen
            Ok(SExpr::List(list))
        }
        Token::Rparen => Err(LpErr::SExpr("Unexpected ')'".to_string())),
        Token::Sym(s) => Ok(SExpr::Sym(s)),
    }
}

fn parse_expr(sexpr: &SExpr) -> Result<Expr, LpErr> {
    match sexpr {
        SExpr::Sym(s) => match s.as_str() {
            "+" | "-" | "*" | "/" => Err(LpErr::Parse(format!("`{}` is not a legal expression", s))),
            _ => match s.parse::<i32>() {
                Ok(n) => Ok(Expr::Num(n)),
                Err(_) => Ok(Expr::Var(s.to_string())),
            }
        }
        SExpr::List(es) => match es.as_slice() {
            [a, SExpr::Sym(op), b] => Ok(Expr::BinaryOp {
                left: Box::new(parse_expr(a)?),
                op: parse_op(op)?,
                right: Box::new(parse_expr(b)?),
            }),
            _ => todo!("invalid code?")
        }
    }
}

fn parse_op(op: &String) -> Result<Operator, LpErr> {
    match op.as_str() {
        "+" => Ok(Operator::Add),
        "-" => Ok(Operator::Sub),
        "*" => Ok(Operator::Mul),
        "/" => Ok(Operator::Div),
        _ => Err(LpErr::Parse(format!("Unknown operator `{}`", op)))
    }
}

mod test {
    use super::*;

    #[test]
    fn tokenize_example_works() -> Result<(), LpErr> {
        let tokens = tokenize("(+ 1 (- 2 3))")?;
        assert_eq!(vec![
            Token::Lparen, Token::Sym("+".to_string()), Token::Sym("1".to_string()),
            Token::Lparen, Token::Sym("-".to_string()), Token::Sym("2".to_string()), Token::Sym("3".to_string()),
            Token::Rparen, Token::Rparen,
        ], tokens);
        Ok(())
    }

    #[test]
    fn parse_simple_sexpr() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + 2)")?;
        let sexpr = parse_sexpr(&mut tokens)?;

        assert_eq!(sexpr, SExpr::List(vec![SExpr::Sym("1".to_string()), SExpr::Sym("+".to_string()), SExpr::Sym("2".to_string())]));

        let mut tokens = tokenize("(1 + a)")?;
        let sexpr = parse_sexpr(&mut tokens)?;

        assert_eq!(sexpr, SExpr::List(vec![SExpr::Sym("1".to_string()), SExpr::Sym("+".to_string()), SExpr::Sym("a".to_string())]));
        Ok(())
    }

    #[test]
    fn parse_complex_sexpr() -> Result<(), LpErr> {
        let mut tokens = tokenize("((1 + 2) asdf :: (a b c))")?;
        let sexpr = parse_sexpr(&mut tokens)?;

        assert_eq!(sexpr, SExpr::List(vec![
            SExpr::List(vec![SExpr::Sym("1".to_string()), SExpr::Sym("+".to_string()), SExpr::Sym("2".to_string())]),
            SExpr::Sym("asdf".to_string()),
            SExpr::Sym("::".to_string()),
            SExpr::List(vec![SExpr::Sym("a".to_string()), SExpr::Sym("b".to_string()), SExpr::Sym("c".to_string())]),
        ]));
        Ok(())
    }

    #[test]
    fn parse_simple_works() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + 2)")?;
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::Num(2)),
        });
        Ok(())
    }

    #[test]
    fn parse_simple_sym() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + a)")?;
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::Var("a".to_string())),
        });
        Ok(())
    }

    #[test]
    fn parse_nested_1() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + (2 * 3))")?;
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Num(2)),
                op: Operator::Mul,
                right: Box::new(Expr::Num(3)),
            }),
        });
        Ok(())
    }

    #[test]
    fn parse_nested_2() -> Result<(), LpErr> {
        let mut tokens = tokenize("((1 + 2) * 3))")?;
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Num(1)),
                op: Operator::Add,
                right: Box::new(Expr::Num(2)),
            }),
            op: Operator::Mul,
            right: Box::new(Expr::Num(3)),
        });
        Ok(())
    }
}
