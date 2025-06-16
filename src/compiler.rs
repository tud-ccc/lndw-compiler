use std::collections::{HashMap, HashSet};
use std::ops::{Add, Div, Mul, Sub};
use std::vec;

use crate::passes::ConstantFold;
pub use crate::types::*;

pub fn compile(input: &String, constant_fold: bool) -> Result<(Vec<Inst>, HashSet<String>), LpErr> {
    let mut ast = parse_expr(&parse_sexpr(tokenize(format!("({})", input)))?)?;
    if constant_fold {
        ast.run_constant_fold();
    }

    generate_ir(&ast)
}

fn tokenize(expr: impl Into<String>) -> Vec<Token> {
    expr.into()
        .replace('(', "( ")
        .replace(')', " )")
        .split_ascii_whitespace()
        .map(|s| match s {
            "(" => Token::Open,
            ")" => Token::Close,
            _ => Token::Sym(s.to_string()),
        })
        .collect::<Vec<Token>>()
}

/// Parse a list of tokens into an s-expression 'tree'. If parentheses are unmatched, this function
/// returns an error.
fn parse_sexpr(mut tokens: Vec<Token>) -> Result<SExpr, LpErr> {
    let result = parse_sexpr_(&mut tokens)?;
    if tokens.is_empty() {
        Ok(result)
    } else {
        Err(LpErr::SExpr(format!("leftover tokens: `{tokens:?}`")))
    }
}

fn parse_sexpr_(tokens: &mut Vec<Token>) -> Result<SExpr, LpErr> {
    match tokens.remove(0) {
        Token::Open => {
            let mut list = Vec::new();
            while !matches!(tokens.first(), Some(Token::Close)) {
                list.push(parse_sexpr_(tokens)?);
                if tokens.is_empty() {
                    return Err(LpErr::SExpr("unclosed list".to_string()));
                }
            }
            assert_eq!(tokens.remove(0), Token::Close); // consume Rparen
            Ok(SExpr::List(list))
        }
        Token::Close => Err(LpErr::SExpr("unexpected ')'".to_string())),
        Token::Sym(s) => Ok(SExpr::Sym(s)),
    }
}

/// Parse an s-expression into an expression tree. Checks for symbol name legality, tries to parse
/// numbers, and extracts valid unary and binary operations.
fn parse_expr(sexpr: &SExpr) -> Result<Expr, LpErr> {
    match sexpr {
        SExpr::Sym(s) => match s.as_str() {
            _ if s.contains(&['+', '-', '*', '/']) => Err(LpErr::Parse(format!(
                "`{}` is not a legal expression or symbol name",
                s
            ))),
            _ => match s.parse::<i32>() {
                Ok(n) => Ok(Expr::Num(n)),
                Err(_) => Ok(Expr::Var(s.to_string())),
            },
        },
        SExpr::List(es) => match es.as_slice() {
            [e @ SExpr::List(..)] => parse_expr(e),
            [SExpr::Sym(op), e] => Ok(Expr::UnaryOp(parse_op(op)?, Box::new(parse_expr(e)?))),
            [a, SExpr::Sym(op), b] => Ok(Expr::BinaryOp(
                Box::new(parse_expr(a)?),
                parse_op(op)?,
                Box::new(parse_expr(b)?),
            )),
            es if es.len() == 4 => Err(LpErr::Parse(format!(
                "`{}` is not a legal expression: too many symbols",
                sexpr
            ))),
            [a, op @ SExpr::Sym(..), b, ts @ ..] => {
                // Try to parse e.g. (a <op> b + 3 ...) into ((a <op> b) + 3 ...)
                let mut tmp = vec![SExpr::List(vec![a.clone(), op.clone(), b.clone()])];
                tmp.extend_from_slice(ts);
                parse_expr(&SExpr::List(tmp))
            }
            _ => Err(LpErr::Parse(format!(
                "`{}` is not a legal expression",
                sexpr
            ))),
        },
    }
}

fn parse_op(op: &String) -> Result<Operator, LpErr> {
    match op.as_str() {
        "+" => Ok(Operator::Add),
        "-" => Ok(Operator::Sub),
        "*" => Ok(Operator::Mul),
        "/" => Ok(Operator::Div),
        _ => Err(LpErr::Parse(format!("unknown operator `{}`", op))),
    }
}

fn generate_ir(ast: &Expr) -> Result<(Vec<Inst>, HashSet<String>), LpErr> {
    let mut reg_counter = 0;
    let mut code: Vec<Inst> = vec![];
    let mut variables = HashSet::new();

    let result_reg = ast_to_ir(ast, &mut reg_counter, &mut code, &mut variables)?;
    code.push(Inst::Result(u8tochar(result_reg)));
    Ok((code, variables))
}

fn u8tochar(reg: u8) -> char {
    char::from_digit(reg as u32 + 10, 36).unwrap()
}

fn ast_to_ir(
    ast: &Expr,
    next_reg: &mut u8,
    code: &mut Vec<Inst>,
    variables: &mut HashSet<String>,
) -> Result<u8, LpErr> {
    match ast {
        Expr::Num(n) => {
            let reg = *next_reg;
            code.push(Inst::Store(*n, u8tochar(reg)));
            *next_reg += 1;
            Ok(reg)
        }
        Expr::Var(v) => {
            let reg = *next_reg; // TODO: avoid duplicate register mapping+transfer
            code.push(Inst::Transfer(v.clone(), u8tochar(reg)));
            variables.insert(v.clone());
            *next_reg += 1;
            Ok(reg)
        }
        Expr::UnaryOp(Operator::Sub, e) => {
            let left_reg = ast_to_ir(&Expr::Num(0), next_reg, code, variables)?;
            let right_reg = ast_to_ir(e, next_reg, code, variables)?;
            code.push(Inst::Sub(u8tochar(left_reg), u8tochar(right_reg)));
            Ok(right_reg)
        }
        Expr::UnaryOp(op, _) => Err(LpErr::IR(format!("invalid unary operator `{op}`"))),
        Expr::BinaryOp(left, op, right) => {
            let left_reg = ast_to_ir(left, next_reg, code, variables)?;
            let right_reg = ast_to_ir(right, next_reg, code, variables)?;

            let inst = match op {
                Operator::Add => Inst::Add(u8tochar(left_reg), u8tochar(right_reg)),
                Operator::Sub => Inst::Sub(u8tochar(left_reg), u8tochar(right_reg)),
                Operator::Mul => Inst::Mul(u8tochar(left_reg), u8tochar(right_reg)),
                Operator::Div => Inst::Div(u8tochar(left_reg), u8tochar(right_reg)),
            };

            code.push(inst);
            Ok(right_reg)
        }
    }
}

pub fn interpret_ir(
    instructions: Vec<Inst>,
    input_variables: &HashMap<String, String>,
) -> Result<i32, LpErr> {
    let mut reg_store = HashMap::<Reg, i32>::new();

    for inst in instructions {
        println!("Variable store is: {reg_store:?}");
        match inst {
            Inst::Add(a, b) => run_binop(a, b, i32::add, &mut reg_store)?,
            Inst::Sub(a, b) => run_binop(a, b, i32::sub, &mut reg_store)?,
            Inst::Mul(a, b) => run_binop(a, b, i32::mul, &mut reg_store)?,
            Inst::Div(a, b) => run_binop(a, b, i32::div, &mut reg_store)?,
            Inst::Store(n, reg) => {
                if reg_store.contains_key(&reg) {
                    eprintln!("Warning: overwriting register `{reg}`.");
                    reg_store.get_mut(&reg).map(|v| *v = n);
                } else {
                    reg_store.insert(reg, n);
                }
            }
            Inst::Transfer(v, _) if !input_variables.contains_key(&v) => {
                return Err(LpErr::Interpret(format!("unknown variable `{v}`")));
            }
            Inst::Transfer(_, r) if reg_store.contains_key(&r) => {
                return Err(LpErr::Interpret(format!(
                    "register `{r}` already contains value"
                )));
            }
            Inst::Transfer(var, reg) => {
                let val_str = input_variables[&var].clone();
                let val = val_str
                    .parse::<i32>()
                    .map_err(|_| LpErr::Interpret(format!("`{val_str}` is not a number")))?;
                reg_store.insert(reg, val);
            }
            Inst::Result(r) => {
                return Ok(*reg_store
                    .get(&r)
                    .ok_or(LpErr::Interpret(format!("register `{}` is empty", r)))?);
            }
        }
    }
    Err(LpErr::Interpret("no result found".to_string()))
}

fn run_binop(
    a: Reg,
    b: Reg,
    op: impl FnOnce(i32, i32) -> i32,
    reg_store: &mut HashMap<Reg, i32>,
) -> Result<(), LpErr> {
    let a = check_store_contains(reg_store, a)?;
    check_store_contains(reg_store, b)?;
    reg_store.get_mut(&b).map(|b| *b = op(a, *b));
    Ok(())
}

fn check_store_contains(store: &HashMap<Reg, i32>, key: Reg) -> Result<i32, LpErr> {
    match store.get(&key) {
        Some(v) => Ok(*v),
        None => Err(LpErr::Interpret(format!("no such reg `{}`", key))),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use SExpr::*;

    #[test]
    fn tokenize_example_works() -> Result<(), LpErr> {
        let tokens = tokenize("(+ 1 (- 2 3))");
        assert_eq!(
            vec![
                Token::Open,
                Token::Sym("+".to_string()),
                Token::Sym("1".to_string()),
                Token::Open,
                Token::Sym("-".to_string()),
                Token::Sym("2".to_string()),
                Token::Sym("3".to_string()),
                Token::Close,
                Token::Close,
            ],
            tokens
        );
        Ok(())
    }

    #[test]
    fn parse_simple_sexpr() -> Result<(), LpErr> {
        let input1 = "(1 + 2)";
        let input2 = "(1 + a)";

        let sexpr = parse_sexpr(tokenize(input1))?;
        assert_eq!(
            sexpr,
            List(vec![
                Sym("1".to_string()),
                Sym("+".to_string()),
                Sym("2".to_string())
            ])
        );

        let sexpr = parse_sexpr(tokenize(input2))?;
        assert_eq!(
            sexpr,
            List(vec![
                Sym("1".to_string()),
                Sym("+".to_string()),
                Sym("a".to_string())
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_complex_sexpr() -> Result<(), LpErr> {
        let sexpr = parse_sexpr(tokenize("((1 + 2) asdf :: (a b c))"))?;

        assert_eq!(
            sexpr,
            List(vec![
                List(vec![
                    Sym("1".to_string()),
                    Sym("+".to_string()),
                    Sym("2".to_string())
                ]),
                Sym("asdf".to_string()),
                Sym("::".to_string()),
                List(vec![
                    Sym("a".to_string()),
                    Sym("b".to_string()),
                    Sym("c".to_string())
                ]),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_invalid_sexpr() -> Result<(), LpErr> {
        let input1 = "((1 + 1)";
        let input2 = "(1 + 1))";

        assert!(parse_sexpr(tokenize(input1)).is_err());
        assert!(parse_sexpr(tokenize(input2)).is_err());
        Ok(())
    }

    #[test]
    fn parse_simple_expr() -> Result<(), LpErr> {
        let expr = parse_expr(&parse_sexpr(tokenize("(1 + 2)"))?)?;

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
        let expr = parse_expr(&parse_sexpr(tokenize("(1 + a)"))?)?;

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
        let expr = parse_expr(&parse_sexpr(tokenize("((((1 + 2))))"))?)?;

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
            "(1 + a/b)",
            "(1 + *)",
            "(1 1 1)",
            "(1 (1) 1)",
            "(1 + 1 1)",
        ];

        for input in inputs {
            assert!(
                parse_expr(&parse_sexpr(tokenize(input))?).is_err(),
                "`{input}` should fail"
            );
        }
        Ok(())
    }

    #[test]
    fn parse_nested_1() -> Result<(), LpErr> {
        let expr = parse_expr(&parse_sexpr(tokenize("(1 + (2 * 3))"))?)?;

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
        let expr = parse_expr(&parse_sexpr(tokenize("((1 + 2) * 3)"))?)?;

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
