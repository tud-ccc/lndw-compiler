use std::collections::{HashMap, HashSet};
use std::ops::{Add, Div, Mul, Sub};
use std::vec;

use crate::parser;
use crate::passes::ConstantFold;
pub use crate::types::*;

pub fn compile(input: &str, constant_fold: bool) -> Result<(Vec<Inst>, HashSet<String>), LpErr> {
    let mut ast = parser::run_parser(input)?;
    if constant_fold {
        ast = ast.run_constant_fold();
    }

    generate_ir(&ast)
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
