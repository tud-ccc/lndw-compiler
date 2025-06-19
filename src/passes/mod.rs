use crate::types::Inst;
use std::collections::HashSet;

mod common_factor_elimination;
mod constant_folding;
mod shift_replacement;

pub use common_factor_elimination::CommonFactorElimination;
pub use constant_folding::ConstantFold;
pub use shift_replacement::ShiftReplacement;

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
