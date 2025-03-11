mod ast;
mod find_redexes;
mod functions;
mod parsing;
mod prelude;
mod types;

pub use ast::*;
pub use find_redexes::{
    find_all_redex_contraction_pairs, find_single_redex_contraction_pair, RCPair,
};
pub use functions::KnownTypeLabelTable;
pub use parsing::{Parser, Token};
pub use prelude::PRELUDE;
pub use types::{typecheck, typecheck_tl_expr, Primitive, Type, TypeError};

#[cfg(test)]
mod lib_test;
