mod find_redexes;
mod functions;
mod parser;
mod types;

pub use find_redexes::find_redex_contraction_pairs;
pub use functions::LabelTable;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use parser::Parser;
pub use types::{typecheck_module, typecheck_tl_expr, Primitive, Type, TypeError};

#[cfg(test)]
mod lib_test;
