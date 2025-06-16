use std::ops::Neg;

use crate::types::{Expr, Operator};

pub trait ConstantFold {
    fn run_constant_fold(&mut self);
}

impl ConstantFold for Expr {
    fn run_constant_fold(&mut self) {
        println!("{self:?}");
        match self {
            Expr::Num(_) | Expr::Var(_) =>
            /* no work to be done */
            {
                ()
            }
            Expr::UnaryOp(operator, expr) => {
                if !expr.is_num() {
                    expr.run_constant_fold();
                }

                if let Expr::Num(n) = expr.as_mut() {
                    if operator == &Operator::Sub {
                        *self = Expr::Num(n.neg());
                    }
                }
            }
            Expr::BinaryOp(lhs, operator, rhs) => {
                if !lhs.is_num() {
                    lhs.run_constant_fold();
                }
                if !rhs.is_num() {
                    rhs.run_constant_fold();
                }

                if let Expr::Num(left) = lhs.as_mut() {
                    if let Expr::Num(right) = rhs.as_mut() {
                        let res = match operator {
                            Operator::Add => *left + *right,
                            Operator::Sub => *left - *right,
                            Operator::Mul => *left * *right,
                            Operator::Div => *left / *right,
                        };
                        *self = res.into();
                    }
                }
            }
        }
    }
}
