pub use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/spec.pest"]
pub struct SflParser;