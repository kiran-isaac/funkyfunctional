use super::*;
use crate::Parser;

#[test]
fn type_check_int_assign() -> Result<(), TypeError> {
    let program = "x :: Int\nx=10\nmain :: Int\nmain = x";

    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut tc = TypeChecker::new();
    tc.check_module(&ast, ast.root)?;

    Ok(())
}

#[test]
fn type_check_const_int_abst() -> Result<(), TypeError> {
    let program = "const_10 :: Float -> Int\nconst_10 = \\x. 10\nmain :: Int\nmain = const_10 2.0";

    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut tc = TypeChecker::new();
    tc.check_module(&ast, ast.root)?;

    Ok(())
}

#[test]
fn type_check_extra_arg_should_fail() {
    let program =
        "const_10 :: Float -> Int\nconst_10 = \\x. 10\nmain :: Int\nmain = const_10 2.0 10";

    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut tc = TypeChecker::new();
    println!("{:?}", tc.check_module(&ast, ast.root).unwrap_err());
}
