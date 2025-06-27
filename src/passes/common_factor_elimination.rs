use crate::types::{Expr, Operator};

pub trait CommonFactorElimination {
    fn extract_common_factors(self) -> Self;
}

impl CommonFactorElimination for Expr {
    fn extract_common_factors(self) -> Self {
        match self {
            Expr::BinaryOp(left, Operator::Add, right) => {
                let left = left.extract_common_factors();
                let right = right.extract_common_factors();

                let common_factors = extract_factors(&left, &right);
                if common_factors.is_empty() {
                    return Expr::BinaryOp(Box::new(left), Operator::Add, Box::new(right));
                }

                let factor = &common_factors[0];
                let left_remainder = remove_factor_from_expr(&left, factor);
                let right_remainder = remove_factor_from_expr(&right, factor);

                let sum = Expr::BinaryOp(
                    Box::new(left_remainder),
                    Operator::Add,
                    Box::new(right_remainder),
                );

                Expr::BinaryOp(Box::new(factor.clone()), Operator::Mul, Box::new(sum))
            }
            Expr::BinaryOp(left, op, right) => {
                let left = left.extract_common_factors();
                let right = right.extract_common_factors();
                Expr::BinaryOp(Box::new(left), op, Box::new(right))
            }
            Expr::UnaryOp(op, expr) => {
                let expr = expr.extract_common_factors();
                Expr::UnaryOp(op, Box::new(expr))
            }
            _ => self,
        }
    }
}

fn extract_factors(left: &Expr, right: &Expr) -> Vec<Expr> {
    let left_factors = get_multiplication_factors(left);
    let right_factors = get_multiplication_factors(right);

    let mut common_factors = Vec::new();

    for left_factor in &left_factors {
        for right_factor in &right_factors {
            if expressions_equal(left_factor, right_factor) {
                common_factors.push(left_factor.clone());
            }
        }
    }

    common_factors
}

fn get_multiplication_factors(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::BinaryOp(left, Operator::Mul, right) => {
            let mut factors = get_multiplication_factors(left);
            factors.extend(get_multiplication_factors(right));
            factors
        }
        _ => vec![expr.clone()],
    }
}

fn expressions_equal(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Num(n1), Expr::Num(n2)) => n1 == n2,
        (Expr::Var(v1), Expr::Var(v2)) => v1 == v2,
        (Expr::UnaryOp(op1, e1), Expr::UnaryOp(op2, e2)) => op1 == op2 && expressions_equal(e1, e2),
        (Expr::BinaryOp(l1, op1, r1), Expr::BinaryOp(l2, op2, r2)) => {
            op1 == op2 && expressions_equal(l1, l2) && expressions_equal(r1, r2)
        }
        _ => false,
    }
}

fn remove_factor_from_expr(expr: &Expr, factor: &Expr) -> Expr {
    match expr {
        Expr::BinaryOp(left, Operator::Mul, right) => {
            if expressions_equal(left, factor) {
                (**right).clone()
            } else if expressions_equal(right, factor) {
                (**left).clone()
            } else {
                let new_left = remove_factor_from_expr(left, factor);
                let new_right = remove_factor_from_expr(right, factor);

                if expressions_equal(&new_left, left) && expressions_equal(&new_right, right) {
                    expr.clone()
                } else {
                    Expr::BinaryOp(Box::new(new_left), Operator::Mul, Box::new(new_right))
                }
            }
        }
        _ => {
            if expressions_equal(expr, factor) {
                Expr::Num(1)
            } else {
                expr.clone()
            }
        }
    }
}
