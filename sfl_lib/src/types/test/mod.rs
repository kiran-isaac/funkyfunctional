use super::*;
use crate::Parser;

#[test]
fn type_check_int_assign() -> Result<(), TypeError> {
    let program = "x :: Int\nx=10\nmain :: Int\nmain = x";

    let ast = Parser::from_string(program.to_string()).parse_module().unwrap() ;
    let mut tc = TypeChecker::new();
    tc.check_module(&ast, ast.root)?;

    Ok(())
}