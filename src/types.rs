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
            LpErr::Parse(e) => write!(f, "{e} (parse)"),
            LpErr::IR(e) => write!(f, "{e} (ir gen)"),
            LpErr::Interpret(e) => write!(f, "{e} (interpreter)"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
pub type MemAddr = usize;

#[derive(Debug, Clone)]
pub enum Inst {
    /// Add two values, storing the result in Register #2.
    Add(Reg, Reg),
    /// Subtract two values, storing the result in Register #2.
    Sub(Reg, Reg),
    /// Multiply two values, storing the result in Register #2.
    Mul(Reg, Reg),
    /// Divide two values, storing the result in Register #2.
    Div(Reg, Reg),
    /// Store a number in a register.
    Store(i32, Reg),
    /// Transfer a value into a register.
    Transfer(String, Reg),
    /// Return the value in the given register and terminate computation.
    Result(Reg),

    /// Write the contents of a register to main memory.
    Write(Reg, MemAddr),
    /// Load a piece of data from main memory into a register.
    Load(MemAddr, Reg),
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
            Inst::Write(r, addr) => write!(f, "write register {r} to main memory (cell {addr})"),
            Inst::Load(addr, r) => write!(f, "load main memory cell {addr} into register {r}"),
        }
    }
}
