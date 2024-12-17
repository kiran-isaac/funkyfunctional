mod parser;
mod find_redexes;
mod inbuilts;
mod types;

pub use parser::Parser;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use find_redexes::get_replacements;
pub use types::{Type, Primitive, TypeError};