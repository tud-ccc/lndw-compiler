use std::collections::{HashMap, HashSet};
use std::ops::{Add, Div, Mul, Sub};
use std::vec;
use rust_i18n::t;
use crate::gui::InterpreterOptions;
use crate::parser;
use crate::passes::{ConstantFold, run_cache_optimization};
pub use crate::types::*;

#[derive(Copy, Clone, Default)]
pub struct CompileOptions {
    pub do_constant_folding: bool,
    pub run_cache_optimization: bool,
}

impl CompileOptions {
    pub fn any(&self) -> bool {
        self.do_constant_folding || self.run_cache_optimization
    }
}

pub struct Compiler {
    options: CompileOptions,
    hw: InterpreterOptions,
}

impl Compiler {
    pub fn with(options: CompileOptions) -> Self {
        Self {
            options,
            hw: Default::default(),
        }
    }

    pub fn with_interpreter(mut self, hw: InterpreterOptions) -> Self {
        self.hw = hw;
        self
    }

    pub fn compile(self, input: &str) -> Result<(Vec<Inst>, HashSet<String>), LpErr> {
        let mut ast = parser::run_parser(input)?;
        if self.options.do_constant_folding {
            ast = ast.run_constant_fold();
        }

        let (mut instructions, variables) = self.generate_ir(&ast)?;

        if self.options.run_cache_optimization {
            instructions = run_cache_optimization(instructions);
        }

        Ok((instructions, variables))
    }

    fn create_write<'a>(
        &self,
        exp: &'a Expr,
        ram_idx: &mut usize,
        code: &mut Vec<Inst>,
        mmap: &mut HashMap<&'a Expr, Location>,
    ) {
        if let Some(val) = mmap.get_mut(exp) {
            if let Location::Reg(r) = val {
                code.push(Inst::Write(u8tochar(*r), *ram_idx));
                *val = Location::Ram(*ram_idx);
                *ram_idx = (*ram_idx + 1) % self.hw.num_cachelines;
                if *ram_idx == 0 {
                    eprintln!("RAM overrun detected");
                }
            } else {
                eprintln!("tried to push RAM to RAM??");
            }
        } else {
            eprintln!("tried to create write for non-existent expression?");
        }
    }

    fn create_load<'a>(
        &self,
        exp: &'a Expr,
        target_reg: &mut u8,
        code: &mut Vec<Inst>,
        mmap: &mut HashMap<&'a Expr, Location>,
    ) {
        if let Some(val) = mmap.get_mut(exp) {
            if let Location::Ram(r) = val {
                code.push(Inst::Load(*r, u8tochar(*target_reg)));
                *val = Location::Reg(*target_reg);
                *target_reg = (*target_reg + 1) % self.hw.num_registers;
            } else {
                eprintln!("tried to load register to register??");
            }
        } else {
            eprintln!("tried to create load for non-existent expression?");
        }
    }

    fn fetch_if_necessary<'a>(
        &self,
        cur_reg: &mut u8,
        e: &'a Expr,
        next_reg: &mut u8,
        ram_idx: &mut usize,
        code: &mut Vec<Inst>,
        mmap: &mut HashMap<&'a Expr, Location>,
        rmap: &mut HashMap<u8, &'a Expr>,
    ) {
        if *rmap.get(cur_reg).unwrap() != e {
            // the entry was evicted -> need a store (maybe) & load
            rmap.entry(*next_reg)
                .and_modify(|expr| {
                    self.create_write(expr, ram_idx, code, mmap);
                    *expr = e;
                })
                .or_insert(e);

            *cur_reg = *next_reg;
            self.create_load(e, next_reg, code, mmap);
        }
    }

    fn ast_to_ir<'a>(
        &self,
        ast: &'a Expr,
        next_reg: &mut u8,
        ram_idx: &mut usize,
        code: &mut Vec<Inst>,
        variables: &mut HashSet<String>,
        mmap: &mut HashMap<&'a Expr, Location>,
        rmap: &mut HashMap<u8, &'a Expr>,
    ) -> Result<u8, LpErr> {
        match ast {
            Expr::Num(n) => {
                let reg = *next_reg;

                // reserve a register for the result and (potentially) evict an existing entry to RAM.
                rmap.entry(reg)
                    .and_modify(|expr| {
                        self.create_write(expr, ram_idx, code, mmap);
                        *expr = ast;
                    })
                    .or_insert(ast);

                code.push(Inst::Store(*n, u8tochar(reg)));
                if mmap.contains_key(ast) {
                    eprintln!(
                        "Tried overwriting existing MMAP value -- duplicate expression {ast:?}"
                    );
                } else {
                    mmap.insert(ast, Location::Reg(reg));
                }

                *next_reg = (*next_reg + 1) % self.hw.num_registers;
                Ok(reg)
            }
            Expr::Var(v) => {
                let reg = *next_reg; // TODO: avoid duplicate register mapping+transfer

                // reserve a register for the result and (potentially) evict an existing entry to RAM.
                rmap.entry(reg)
                    .and_modify(|expr| {
                        self.create_write(expr, ram_idx, code, mmap);
                        *expr = ast;
                    })
                    .or_insert(ast);

                code.push(Inst::Transfer(v.clone(), u8tochar(reg)));
                if mmap.contains_key(ast) {
                    eprintln!(
                        "[warn] Tried overwriting existing MMAP value -- duplicate expression {ast:?}"
                    );
                } else {
                    mmap.insert(ast, Location::Reg(reg));
                }

                variables.insert(v.clone());
                *next_reg = (*next_reg + 1) % self.hw.num_registers;
                Ok(reg)
            }
            Expr::UnaryOp(Operator::Sub, e) => {
                // TODO: optimization potential -> do the right register first to avoid collisions
                let mut left_reg = self.ast_to_ir(
                    &Expr::Num(0),
                    next_reg,
                    ram_idx,
                    code,
                    variables,
                    mmap,
                    rmap,
                )?;
                let mut right_reg =
                    self.ast_to_ir(e, next_reg, ram_idx, code, variables, mmap, rmap)?;

                self.fetch_if_necessary(
                    &mut left_reg,
                    &Expr::Num(0),
                    next_reg,
                    ram_idx,
                    code,
                    mmap,
                    rmap,
                );

                self.fetch_if_necessary(&mut right_reg, e, next_reg, ram_idx, code, mmap, rmap);

                code.push(Inst::Sub(u8tochar(left_reg), u8tochar(right_reg)));

                rmap.entry(right_reg).and_modify(|val| *val = ast);
                // forced insert here because register is more useful than a potential hit in RAM
                mmap.insert(ast, Location::Reg(right_reg));

                Ok(right_reg)
            }
            Expr::UnaryOp(op, _) => Err(LpErr::IR(format!("invalid unary operator `{op}`"))),
            Expr::BinaryOp(left, op, right) => {
                let mut left_reg =
                    self.ast_to_ir(left, next_reg, ram_idx, code, variables, mmap, rmap)?;
                let mut right_reg =
                    self.ast_to_ir(right, next_reg, ram_idx, code, variables, mmap, rmap)?;

                self.fetch_if_necessary(&mut left_reg, left, next_reg, ram_idx, code, mmap, rmap);
                self.fetch_if_necessary(&mut right_reg, right, next_reg, ram_idx, code, mmap, rmap);

                let inst = match op {
                    Operator::Add => Inst::Add(u8tochar(left_reg), u8tochar(right_reg)),
                    Operator::Sub => Inst::Sub(u8tochar(left_reg), u8tochar(right_reg)),
                    Operator::Mul => Inst::Mul(u8tochar(left_reg), u8tochar(right_reg)),
                    Operator::Div => Inst::Div(u8tochar(left_reg), u8tochar(right_reg)),
                };

                code.push(inst);

                rmap.entry(right_reg).and_modify(|val| *val = ast);
                // forced insert here because register is more useful than a potential hit in RAM
                mmap.insert(ast, Location::Reg(right_reg));

                Ok(right_reg)
            }
        }
    }

    fn generate_ir(&self, ast: &Expr) -> Result<(Vec<Inst>, HashSet<String>), LpErr> {
        let mut reg_counter = 0;
        let mut ram_idx = 0;
        let mut code: Vec<Inst> = vec![];
        let mut variables = HashSet::new();

        let mut mmap = HashMap::new();
        let mut rmap = HashMap::new();

        let result_reg = self.ast_to_ir(
            ast,
            &mut reg_counter,
            &mut ram_idx,
            &mut code,
            &mut variables,
            &mut mmap,
            &mut rmap,
        )?;
        code.push(Inst::Result(u8tochar(result_reg)));
        Ok((code, variables))
    }
}

pub fn interpret_ir(
    instructions: Vec<&Inst>,
    input_variables: &HashMap<String, String>,
    hw: InterpreterOptions,
) -> Result<i32, LpErr> {
    let mut reg_store = HashMap::<Reg, i32>::new();
    let mut ram = vec![0; hw.num_cachelines];

    for inst in instructions {
        println!("Variable store is: {reg_store:?}");
        match inst {
            Inst::Add(a, b) => run_binop(*a, *b, i32::add, &mut reg_store)?,
            Inst::Sub(a, b) => run_binop(*a, *b, i32::sub, &mut reg_store)?,
            Inst::Mul(a, b) => run_binop(*a, *b, i32::mul, &mut reg_store)?,
            Inst::Div(a, b) => {
                if check_store_contains(&reg_store, *b)? == 0 {
                    return Err(LpErr::Interpret(t!("compiler.error.divzero").to_string()));
                }
                run_binop(*a, *b, i32::div, &mut reg_store)?
            }
            Inst::Store(n, reg) => {
                if reg_store.contains_key(reg) {
                    eprintln!("Warning: overwriting register `{reg}`.");
                    if let Some(v) = reg_store.get_mut(reg) {
                        *v = *n;
                    }
                } else {
                    reg_store.insert(*reg, *n);
                }
            }
            Inst::Transfer(v, _) if !input_variables.contains_key(v) => {
                return Err(LpErr::Interpret(t!("compiler.error.unkownvar",  v = v).to_string()));
            }
            Inst::Transfer(_, r) if reg_store.contains_key(r) => {
                return Err(LpErr::Interpret(format!(
                    "register `{r}` already contains value"
                )));
            }
            Inst::Transfer(var, reg) => {
                let val_str = input_variables[var].clone();
                let val = val_str
                    .parse::<i32>()
                    .map_err(|_| LpErr::Interpret(format!("`{val_str}` is not a number")))?;
                reg_store.insert(*reg, val);
            }
            Inst::Result(r) => {
                return Ok(*reg_store
                    .get(r)
                    .ok_or(LpErr::Interpret(format!("register `{r}` is empty")))?);
            }
            Inst::Write(_, addr) | Inst::Load(addr, _) if addr >= &ram.len() => {
                return Err(LpErr::Interpret(format!(
                    "requested RAM address {addr} doesn't exist."
                )));
            }
            Inst::Write(r, addr) => {
                if let Some(val) = reg_store.get(r) {
                    ram[*addr] = *val;
                } else {
                    return Err(LpErr::Interpret(format!("register `{r}` is empty")));
                }
            }
            Inst::Load(addr, r) => {
                reg_store
                    .entry(*r)
                    .and_modify(|e| *e = ram[*addr])
                    .or_insert(ram[*addr]);
            }
        }
    }
    Err(LpErr::Interpret("no result found".to_string()))
}

fn u8tochar(reg: u8) -> char {
    char::from_digit(reg as u32 + 10, 36).unwrap()
}

/// Describes a memory address either as register or RAM address
pub enum Location {
    Ram(MemAddr),
    Reg(u8),
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

fn check_store_contains(store: &HashMap<Reg, i32>, key: Reg) -> Result<i32, LpErr> {
    match store.get(&key) {
        Some(v) => Ok(*v),
        None => Err(LpErr::Interpret(format!("no such reg `{key}`"))),
    }
}
