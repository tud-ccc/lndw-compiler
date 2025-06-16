use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LpErr {
    Parse(String),
    IR(String),
    Interpret(String),
}

impl Display for LpErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LpErr::Parse(e) => write!(f, "{} (parse)", e),
            LpErr::IR(e) => write!(f, "{} (ir gen)", e),
            LpErr::Interpret(e) => write!(f, "{} (interpreter)", e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl TryFrom<char> for Operator {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Sub),
            '*' => Ok(Operator::Mul),
            '/' => Ok(Operator::Div),
            _ => Err(value),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
        }
    }
}

/// The main AST struct for representing the IR.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Num(i32),
    Var(String),
    UnaryOp(Operator, Box<Expr>),
    BinaryOp(Box<Expr>, Operator, Box<Expr>),
}

impl Expr {
    pub fn is_num(&self) -> bool {
        matches!(self, Expr::Num { .. })
    }
}

impl From<i32> for Expr {
    fn from(value: i32) -> Self {
        Expr::Num(value)
    }
}

pub type Reg = char;

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
