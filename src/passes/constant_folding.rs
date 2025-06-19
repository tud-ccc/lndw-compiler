use std::ops::Neg;

use crate::types::{Expr, Operator};

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

                if let Expr::Num(n) = e
                    && operator == Operator::Sub
                {
                    return Expr::Num(n.neg());
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

                if let Expr::Num(left) = l
                    && let Expr::Num(right) = r
                {
                    let res = match operator {
                        Operator::Add => left + right,
                        Operator::Sub => left - right,
                        Operator::Mul => left * right,
                        Operator::Div => {
                            if right == 0 {
                                return Expr::BinaryOp(Box::new(l), operator, Box::new(r))
                            }
                            left / right
                        },
                    };
                    return res.into();
                }

                Expr::BinaryOp(Box::new(l), operator, Box::new(r))
            }
        }
    }
}
