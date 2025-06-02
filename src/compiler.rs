use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::vec;

#[derive(Debug)]
pub enum LpErr {
    SExpr(String),
    Parse(String),
    IR(String),
    Interpret(String),
}

impl Display for LpErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LpErr::SExpr(e) => write!(f, "{} (s-expr)", e),
            LpErr::Parse(e) => write!(f, "{} (parse)", e),
            LpErr::IR(e) => write!(f, "{} (ir gen)", e),
            LpErr::Interpret(e) => write!(f, "{} (interpreter)", e),
        }
    }
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
    /// Opening parenthesis `(`.
    Open,
    /// Closing parenthesis `)`.
    Close,
    /// Any symbol, e.g. `1`, `a`, `+`, `asdf`, `_#z1+`, that is not a parenthesis.
    Sym(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SExpr {
    Sym(String),
    List(Vec<SExpr>),
}

impl Display for SExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SExpr::Sym(e) => write!(f, "{}", e),
            SExpr::List(es) => {
                write!(f, "(")?;
                for (count, v) in es.iter().enumerate() {
                    if count != 0 { write!(f, " ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
        }
    }
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

pub type Reg = u8;

#[derive(Debug, Clone)]
pub enum Inst {
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Mul(Reg, Reg),
    Div(Reg, Reg),
    Store(i32, Reg),
    Transfer(String, Reg),
    Result(Reg),
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Inst::Add(a, b) => write!(f, "add register {a} to register {b}"),
            Inst::Sub(a, b) => write!(f, "subtract register {a} from register {b}"),
            Inst::Mul(a, b) => write!(f, "multiply register {a} by register {b}"),
            Inst::Div(a, b) => write!(f, "divide register {a} by register {b}"),
            Inst::Store(n, r) => write!(f, "store the number {n} in register {r}"),
            Inst::Transfer(v, r) => write!(f, "transfer variable {v} to register {r}"),
            Inst::Result(r) => write!(f, "the result is in register {r}"),
        }
    }
}

pub fn compile(input: &String) -> Result<(Vec<Inst>, HashMap<String, Reg>), LpErr> {
    let ast = parse_expr(&parse_sexpr(&mut tokenize(format!("({})", input)))?)?;
    generate_ir(&ast)
}

fn tokenize(expr: impl Into<String>) -> Vec<Token> {
    expr.into().replace('(', "( ")
        .replace(')', " )")
        .split_ascii_whitespace()
        .map(|s| match s {
            "(" => Token::Open,
            ")" => Token::Close,
            _ => Token::Sym(s.to_string()),
        })
        .collect::<Vec<Token>>()
}

fn parse_sexpr(tokens: &mut Vec<Token>) -> Result<SExpr, LpErr> {
    match tokens.remove(0) {
        Token::Open => {
            let mut list = Vec::new();
            while !matches!(tokens.first(), Some(Token::Close)) {
                list.push(parse_sexpr(tokens)?);
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

fn parse_expr(sexpr: &SExpr) -> Result<Expr, LpErr> {
    match sexpr {
        SExpr::Sym(s) => match s.as_str() {
            _ if s.contains(&['+', '-', '*', '/']) => Err(LpErr::Parse(format!("`{}` is not a legal expression or symbol name", s))),
            _ => match s.parse::<i32>() {
                Ok(n) => Ok(Expr::Num(n)),
                Err(_) => Ok(Expr::Var(s.to_string())),
            }
        }
        SExpr::List(es) => match es.as_slice() {
            [e @ SExpr::List(..)] => parse_expr(e),
            [a, SExpr::Sym(op), b] => Ok(Expr::BinaryOp {
                left: Box::new(parse_expr(a)?),
                op: parse_op(op)?,
                right: Box::new(parse_expr(b)?),
            }),
            es if es.len() == 4 => Err(LpErr::Parse(format!("`{}` is not a legal expression: too many symbols", sexpr))),
            [a, op @ SExpr::Sym(..), b, ts @ ..] => {
                // Try to parse e.g. (a <op> b + 3 ...) into ((a <op> b) + 3 ...)
                let mut tmp = vec![SExpr::List(vec![a.clone(), op.clone(), b.clone()])];
                tmp.extend_from_slice(ts);
                parse_expr(&SExpr::List(tmp))
            },
            _ => Err(LpErr::Parse(format!("`{}` is not a legal expression: second entry must be an op", sexpr))),
        }
    }
}

fn parse_op(op: &String) -> Result<Operator, LpErr> {
    match op.as_str() {
        "+" => Ok(Operator::Add),
        "-" => Ok(Operator::Sub),
        "*" => Ok(Operator::Mul),
        "/" => Ok(Operator::Div),
        _ => Err(LpErr::Parse(format!("unknown operator `{}`", op)))
    }
}

fn generate_ir(ast: &Expr) -> Result<(Vec<Inst>, HashMap<String, Reg>), LpErr> {
    let mut reg_counter = 0;
    let mut code: Vec<Inst> = vec![];
    let mut variables = HashMap::new();

    let result_reg = ast_to_ir(ast, &mut reg_counter, &mut code, &mut variables)?;
    code.push(Inst::Result(result_reg));
    Ok((code, variables))
}

fn ast_to_ir(ast: &Expr, next_reg: &mut u8, code: &mut Vec<Inst>, variables: &mut HashMap<String, Reg>) -> Result<u8, LpErr> {
    match ast {
        Expr::Num(n) => {
            let reg = *next_reg;
            code.push(Inst::Store(*n, reg));
            *next_reg += 1;
            Ok(reg)
        }
        Expr::Var(v) => {
            let reg = *next_reg; // TODO: avoid duplicate register mapping+transfer
            code.push(Inst::Transfer(v.clone(), reg));
            variables.insert(v.clone(), reg);
            *next_reg += 1;
            Ok(reg)
        }
        Expr::BinaryOp { left, op, right } => {
            let left_reg = ast_to_ir(left, next_reg, code, variables)?;
            let right_reg = ast_to_ir(right, next_reg, code, variables)?;

            let inst = match op {
                Operator::Add => Inst::Add(left_reg, right_reg),
                Operator::Sub => Inst::Sub(left_reg, right_reg),
                Operator::Mul => Inst::Mul(left_reg, right_reg),
                Operator::Div => Inst::Div(left_reg, right_reg),
            };

            code.push(inst);
            Ok(right_reg)
        }
    }
}

pub fn interpret_ir(instructions: Vec<Inst>, variable_mapping: &HashMap<String, (Reg, String)>) -> Result<i32, LpErr> {
    let mut reg_store: HashMap<Reg, i32> = variable_mapping.iter()
        .map(|(var, (reg, val))| val.parse::<i32>().map(|v| (*reg, v)).map_err(|_| LpErr::Interpret(format!("couldn't interpret `{val}` as number"))))
        .collect::<Result<_, _>>()?;

    for inst in instructions {
        println!("Variable store is: {reg_store:?}");
        match inst {
            Inst::Add(a, b) => {
                let a = check_store_contains(&reg_store, a)?;
                check_store_contains(&reg_store, b)?;
                reg_store.get_mut(&b).map(|b| *b = a + *b);
            }
            Inst::Sub(a, b) => {
                let a = check_store_contains(&reg_store, a)?;
                check_store_contains(&reg_store, b)?;
                reg_store.get_mut(&b).map(|b| *b = a - *b);
            }
            Inst::Mul(a, b) => {
                let a = check_store_contains(&reg_store, a)?;
                check_store_contains(&reg_store, b)?;
                reg_store.get_mut(&b).map(|b| *b = a * *b);
            }
            Inst::Div(a, b) => {
                let a = check_store_contains(&reg_store, a)?;
                check_store_contains(&reg_store, b)?;
                reg_store.get_mut(&b).map(|b| *b = a / *b);
            }
            Inst::Store(n, reg) => if reg_store.contains_key(&reg) {
                reg_store.get_mut(&reg).map(|_| n);
            } else {
                reg_store.insert(reg, n);
            }
            Inst::Transfer(v, reg) => {}
            Inst::Result(r) => {
                return Ok(*reg_store.get(&r).ok_or(LpErr::Interpret(format!("register `{}` is empty", r)))?);
            }
        }
    }
    Err(LpErr::Interpret("no result found".to_string()))
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
        assert_eq!(vec![
            Token::Open, Token::Sym("+".to_string()), Token::Sym("1".to_string()),
            Token::Open, Token::Sym("-".to_string()), Token::Sym("2".to_string()), Token::Sym("3".to_string()),
            Token::Close, Token::Close,
        ], tokens);
        Ok(())
    }

    #[test]
    fn parse_simple_sexpr() -> Result<(), LpErr> {
        let input1 = "(1 + 2)";
        let input2 = "(1 + a)";

        let sexpr = parse_sexpr(&mut tokenize(input1))?;
        assert_eq!(sexpr, List(vec![Sym("1".to_string()), Sym("+".to_string()), Sym("2".to_string())]));

        let sexpr = parse_sexpr(&mut tokenize(input2))?;
        assert_eq!(sexpr, List(vec![Sym("1".to_string()), Sym("+".to_string()), Sym("a".to_string())]));
        Ok(())
    }

    #[test]
    fn parse_complex_sexpr() -> Result<(), LpErr> {
        let mut tokens = tokenize("((1 + 2) asdf :: (a b c))");
        let sexpr = parse_sexpr(&mut tokens)?;

        assert_eq!(sexpr, List(vec![
            List(vec![Sym("1".to_string()), Sym("+".to_string()), Sym("2".to_string())]),
            Sym("asdf".to_string()),
            Sym("::".to_string()),
            List(vec![Sym("a".to_string()), Sym("b".to_string()), Sym("c".to_string())]),
        ]));
        Ok(())
    }

    #[test]
    fn parse_simple_expr() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + 2)");
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
        let mut tokens = tokenize("(1 + a)");
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::Var("a".to_string())),
        });
        Ok(())
    }

    #[test]
    fn parse_nested_parens() -> Result<(), LpErr> {
        let mut tokens = tokenize("((((1 + 2))))");
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::Num(2)),
        });
        Ok(())
    }

    #[test]
    fn parse_invalid_expr() -> Result<(), LpErr> {
        let input1 = "(1 + 2+)";
        let input2 = "(1 + +2)";
        let input3 = "(1 + a/b)";
        let input4 = "(1 + *)";
        let input5 = "(1 1 1)";
        let input6 = "(1 (1) 1)";
        let input7 = "(1 + 1 1)";

        assert!(parse_expr(&parse_sexpr(&mut tokenize(input1))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input2))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input3))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input4))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input5))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input6))?).is_err());
        assert!(parse_expr(&parse_sexpr(&mut tokenize(input7))?).is_err());
        Ok(())
    }

    #[test]
    fn parse_nested_1() -> Result<(), LpErr> {
        let mut tokens = tokenize("(1 + (2 * 3))");
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::Num(1)),
            op: Operator::Add,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Num(2)), op: Operator::Mul, right: Box::new(Expr::Num(3))
            }),
        });
        Ok(())
    }

    #[test]
    fn parse_nested_2() -> Result<(), LpErr> {
        let mut tokens = tokenize("((1 + 2) * 3))");
        let expr = parse_expr(&parse_sexpr(&mut tokens)?)?;

        assert_eq!(expr, Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Num(1)), op: Operator::Add, right: Box::new(Expr::Num(2))
            }),
            op: Operator::Mul,
            right: Box::new(Expr::Num(3)),
        });
        Ok(())
    }
}
