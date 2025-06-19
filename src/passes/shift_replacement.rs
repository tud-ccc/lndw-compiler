use crate::types::{Expr, Operator};

pub trait ShiftReplacement {
    fn replace_multiplications_with_bitshifts(self) -> Self;
}

impl ShiftReplacement for Expr {
    fn replace_multiplications_with_bitshifts(self) -> Self {
        match self {
            Expr::Num(_) | Expr::Var(_) => self,
            Expr::UnaryOp(op, expr) => {
                Expr::UnaryOp(op, Box::new(expr.replace_multiplications_with_bitshifts()))
            }
            Expr::BinaryOp(
                lhs,
                o @ (Operator::Add | Operator::Sub | Operator::Shl | Operator::Shr),
                rhs,
            ) => Expr::BinaryOp(
                Box::new(lhs.replace_multiplications_with_bitshifts()),
                o,
                Box::new(rhs.replace_multiplications_with_bitshifts()),
            ),
            Expr::BinaryOp(left, Operator::Mul, right) => {
                // this is actually a neat trick since 1000 & 0111 == 0 and that holds true for all powers of 2
                if let &Expr::Num(lhs) = left.as_ref()
                    && (lhs & (lhs - 1)) == 0
                {
                    Expr::BinaryOp(
                        right,
                        Operator::Shl,
                        Box::new(Expr::Num(lhs.ilog2() as i32)),
                    )
                } else if let &Expr::Num(rhs) = right.as_ref()
                    && (rhs & (rhs - 1)) == 0
                {
                    Expr::BinaryOp(
                        Box::new(left.replace_multiplications_with_bitshifts()),
                        Operator::Shl,
                        Box::new(Expr::Num(rhs.ilog2() as i32)),
                    )
                } else {
                    Expr::BinaryOp(
                        Box::new(left.replace_multiplications_with_bitshifts()),
                        Operator::Mul,
                        Box::new(right.replace_multiplications_with_bitshifts()),
                    )
                }
            }
            Expr::BinaryOp(left, Operator::Div, right) => {
                if let &Expr::Num(rhs) = right.as_ref()
                    && (rhs & (rhs - 1)) == 0
                {
                    Expr::BinaryOp(
                        Box::new(left.replace_multiplications_with_bitshifts()),
                        Operator::Shr,
                        Box::new(Expr::Num(rhs.ilog2() as i32)),
                    )
                } else {
                    Expr::BinaryOp(
                        Box::new(left.replace_multiplications_with_bitshifts()),
                        Operator::Div,
                        Box::new(right.replace_multiplications_with_bitshifts()),
                    )
                }
            }
        }
    }
}
