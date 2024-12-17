mod parser;
mod find_redexes;
mod inbuilts;

pub use parser::Parser;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use find_redexes::get_replacements;