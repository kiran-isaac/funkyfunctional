pub mod token;
pub use pest::Parser;
pub mod lexer;

#[derive(Parser)]
#[grammar = "parser/spec.pest"]
pub struct SflParser;