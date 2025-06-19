use rust_i18n::t;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

use crate::{
    gui::InterpreterOptions,
    types::{Inst, LpErr, Reg},
};

/// State of the interpreter after executing a single execution step.
pub enum InterpreterState {
    /// Continue execution with the next instruction.
    Continue,
    /// The execution terminated successfully.
    Finished(i32),
}

impl From<i32> for InterpreterState {
    fn from(value: i32) -> Self {
        InterpreterState::Finished(value)
    }
}

/// Interpreter for our custom ISA.
///
/// The interpreters stores the memory layout at each step and thus enables introspection.
pub struct Interpreter<'a> {
    /// The register store.
    pub reg_store: HashMap<Reg, i32>,
    /// Slow cache used for out-of-register storage.
    pub ram: Vec<i32>,

    /// Instruction list to be executed.
    instructions: Vec<&'a Inst>,
    /// Program counter pointing to the next instruction to be executed.
    program_counter: usize,

    /// Input variable mapping.
    input_variables: Option<&'a HashMap<String, String>>,
}

impl<'a> Interpreter<'a> {
    /// Instantiates a new interpreter with the given hardware configuration.
    pub fn with_config(hw: &InterpreterOptions) -> Self {
        Self {
            reg_store: Default::default(),
            ram: vec![0; hw.num_cachelines],
            instructions: Vec::with_capacity(0),
            program_counter: 0,
            input_variables: None,
        }
    }

    /// Loads a list of instructions into the interpreter.
    pub fn load_instructions(mut self, instructions: Vec<&'a Inst>) -> Self {
        self.instructions = instructions;
        self
    }

    /// Maps inputs to variables.
    pub fn with_variables(mut self, input_variables: &'a HashMap<String, String>) -> Self {
        self.input_variables = Some(input_variables);
        self
    }

    /// Executes the instruction list until the interpreter either terminates or encounters a critical error.
    pub fn run_to_end(mut self) -> Result<i32, LpErr> {
        loop {
            match self.step()? {
                InterpreterState::Continue => (),
                InterpreterState::Finished(res) => return Ok(res),
            }
        }
    }

    /// Executes a single step of the program.
    pub fn step(&mut self) -> Result<InterpreterState, LpErr> {
        println!("Variable store is: {:?}", self.reg_store);
        match self.instructions[self.program_counter] {
            Inst::Add(a, b) => run_binop(*a, *b, i32::add, &mut self.reg_store)?,
            Inst::Sub(a, b) => run_binop(*a, *b, i32::sub, &mut self.reg_store)?,
            Inst::Mul(a, b) => run_binop(*a, *b, i32::mul, &mut self.reg_store)?,
            Inst::Div(a, b) => {
                if check_store_contains(&self.reg_store, *b)? == 0 {
                    return Err(LpErr::Interpret(t!("compiler.error.divzero").to_string()));
                }
                run_binop(*a, *b, i32::div, &mut self.reg_store)?
            }
            Inst::Shl(a, b) => run_shiftop(*a, *b, i32::unbounded_shl, &mut self.reg_store)?,
            Inst::Shr(a, b) => run_shiftop(*a, *b, i32::unbounded_shr, &mut self.reg_store)?,
            Inst::Store(n, reg) => {
                if self.reg_store.contains_key(reg) {
                    eprintln!("Warning: overwriting register `{reg}`.");
                    if let Some(v) = self.reg_store.get_mut(reg) {
                        *v = *n;
                    }
                } else {
                    self.reg_store.insert(*reg, *n);
                }
            }
            Inst::Transfer(v, _)
                if !self
                    .input_variables
                    .ok_or(LpErr::Interpret("No variables loaded".into()))?
                    .contains_key(v) =>
            {
                return Err(LpErr::Interpret(
                    t!("compiler.error.unkownvar", v = v).to_string(),
                ));
            }
            Inst::Transfer(_, r) if self.reg_store.contains_key(r) => {
                return Err(LpErr::Interpret(format!(
                    "register `{r}` already contains value"
                )));
            }
            Inst::Transfer(var, reg) => {
                let val_str = self
                    .input_variables
                    .ok_or(LpErr::Interpret("No variables loaded".into()))?[var]
                    .clone();
                let val = val_str
                    .parse::<i32>()
                    .map_err(|_| LpErr::Interpret(format!("`{val_str}` is not a number")))?;
                self.reg_store.insert(*reg, val);
            }
            Inst::Result(r) => {
                return Ok((*self
                    .reg_store
                    .get(r)
                    .ok_or(LpErr::Interpret(format!("register `{r}` is empty")))?)
                .into());
            }
            Inst::Write(_, addr) | Inst::Load(addr, _) if addr >= &self.ram.len() => {
                return Err(LpErr::Interpret(format!(
                    "requested RAM address {addr} doesn't exist."
                )));
            }
            Inst::Write(r, addr) => {
                if let Some(val) = self.reg_store.get(r) {
                    self.ram[*addr] = *val;
                } else {
                    return Err(LpErr::Interpret(format!("register `{r}` is empty")));
                }
            }
            Inst::Load(addr, r) => {
                self.reg_store
                    .entry(*r)
                    .and_modify(|e| *e = self.ram[*addr])
                    .or_insert(self.ram[*addr]);
            }
        }
        // Err(LpErr::Interpret("no result found".to_string()))

        self.program_counter += 1;
        Ok(InterpreterState::Continue)
    }
}

fn run_binop(
    a: Reg,
    b: Reg,
    op: impl FnOnce(i32, i32) -> i32,
    reg_store: &mut HashMap<Reg, i32>,
) -> Result<(), LpErr> {
    let a = check_store_contains(reg_store, a)?;
    check_store_contains(reg_store, b)?;
    if let Some(b) = reg_store.get_mut(&b) {
        *b = op(a, *b);
    }
    Ok(())
}

fn run_shiftop(
    a: Reg,
    b: Reg,
    op: impl FnOnce(i32, u32) -> i32,
    reg_store: &mut HashMap<Reg, i32>,
) -> Result<(), LpErr> {
    let a = check_store_contains(reg_store, a)?;
    check_store_contains(reg_store, b)?;

    if let Some(b) = reg_store.get_mut(&b) {
        *b = op(a, *b as u32);
    }
    Ok(())
}

fn check_store_contains(store: &HashMap<Reg, i32>, key: Reg) -> Result<i32, LpErr> {
    match store.get(&key) {
        Some(v) => Ok(*v),
        None => Err(LpErr::Interpret(format!("no such reg `{key}`"))),
    }
}

// pub fn interpret_ir(
//     instructions: Vec<&Inst>,
//     input_variables: &HashMap<String, String>,
//     hw: InterpreterOptions,
// ) -> Result<i32, LpErr> {
//     let mut reg_store = HashMap::<Reg, i32>::new();
//     let mut ram = vec![0; hw.num_cachelines];

//     for inst in instructions {
//         println!("Variable store is: {reg_store:?}");
//         match inst {
//             Inst::Add(a, b) => run_binop(*a, *b, i32::add, &mut reg_store)?,
//             Inst::Sub(a, b) => run_binop(*a, *b, i32::sub, &mut reg_store)?,
//             Inst::Mul(a, b) => run_binop(*a, *b, i32::mul, &mut reg_store)?,
//             Inst::Div(a, b) => {
//                 if check_store_contains(&reg_store, *b)? == 0 {
//                     return Err(LpErr::Interpret(t!("compiler.error.divzero").to_string()));
//                 }
//                 run_binop(*a, *b, i32::div, &mut reg_store)?
//             }
//             Inst::Shl(a, b) => run_shiftop(*a, *b, i32::unbounded_shl, &mut reg_store)?,
//             Inst::Shr(a, b) => run_shiftop(*a, *b, i32::unbounded_shr, &mut reg_store)?,
//             Inst::Store(n, reg) => {
//                 if reg_store.contains_key(reg) {
//                     eprintln!("Warning: overwriting register `{reg}`.");
//                     if let Some(v) = reg_store.get_mut(reg) {
//                         *v = *n;
//                     }
//                 } else {
//                     reg_store.insert(*reg, *n);
//                 }
//             }
//             Inst::Transfer(v, _) if !input_variables.contains_key(v) => {
//                 return Err(LpErr::Interpret(
//                     t!("compiler.error.unkownvar", v = v).to_string(),
//                 ));
//             }
//             Inst::Transfer(_, r) if reg_store.contains_key(r) => {
//                 return Err(LpErr::Interpret(format!(
//                     "register `{r}` already contains value"
//                 )));
//             }
//             Inst::Transfer(var, reg) => {
//                 let val_str = input_variables[var].clone();
//                 let val = val_str
//                     .parse::<i32>()
//                     .map_err(|_| LpErr::Interpret(format!("`{val_str}` is not a number")))?;
//                 reg_store.insert(*reg, val);
//             }
//             Inst::Result(r) => {
//                 return Ok(*reg_store
//                     .get(r)
//                     .ok_or(LpErr::Interpret(format!("register `{r}` is empty")))?);
//             }
//             Inst::Write(_, addr) | Inst::Load(addr, _) if addr >= &ram.len() => {
//                 return Err(LpErr::Interpret(format!(
//                     "requested RAM address {addr} doesn't exist."
//                 )));
//             }
//             Inst::Write(r, addr) => {
//                 if let Some(val) = reg_store.get(r) {
//                     ram[*addr] = *val;
//                 } else {
//                     return Err(LpErr::Interpret(format!("register `{r}` is empty")));
//                 }
//             }
//             Inst::Load(addr, r) => {
//                 reg_store
//                     .entry(*r)
//                     .and_modify(|e| *e = ram[*addr])
//                     .or_insert(ram[*addr]);
//             }
//         }
//     }
//     Err(LpErr::Interpret("no result found".to_string()))
// }
