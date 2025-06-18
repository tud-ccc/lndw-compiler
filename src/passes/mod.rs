use std::{collections::HashSet, ops::Neg};

use crate::types::{Expr, Inst, Operator};

pub trait ConstantFold {
    fn run_constant_fold(self) -> Self;
}

impl ConstantFold for Expr {
    fn run_constant_fold(self) -> Self {
        match self {
            Expr::Num(_) | Expr::Var(_) =>
            /* no work to be done */
            {
                self
            }
            Expr::UnaryOp(operator, expr) => {
                let e = if !expr.is_num() {
                    expr.run_constant_fold()
                } else {
                    *expr
                };

                if let Expr::Num(n) = e {
                    if operator == Operator::Sub {
                        return Expr::Num(n.neg());
                    }
                }

                Expr::UnaryOp(operator, Box::new(e))
            }
            Expr::BinaryOp(lhs, operator, rhs) => {
                let l = if !lhs.is_num() {
                    lhs.run_constant_fold()
                } else {
                    *lhs
                };

                let r = if !rhs.is_num() {
                    rhs.run_constant_fold()
                } else {
                    *rhs
                };

                if let Expr::Num(left) = l {
                    if let Expr::Num(right) = r {
                        let res = match operator {
                            Operator::Add => left + right,
                            Operator::Sub => left - right,
                            Operator::Mul => left * right,
                            Operator::Div => left / right,
                        };
                        return res.into();
                    }
                }

                Expr::BinaryOp(Box::new(l), operator, Box::new(r))
            }
        }
    }
}

/// Remove cache writes of lines that are never loaded
pub fn run_cache_optimization(instructions: Vec<Inst>) -> Vec<Inst> {
    let loaded_lines: HashSet<usize> = instructions
        .iter()
        .filter_map(|i| match i {
            Inst::Load(addr, _) => Some(*addr),
            _ => None,
        })
        .collect();

    instructions
        .into_iter()
        .filter(|i| match i {
            Inst::Write(_, target_addr) => loaded_lines.contains(target_addr),
            _ => true,
        })
        .collect()
}
