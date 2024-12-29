use std::collections::HashMap;

use ast::AST;

use crate::parser::*;

#[test]
fn assign() -> Result<(), ParserError> {
    let str = "x = add 2 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_assign_exp(assign);

    let left = ast.get_func(exp);
    let right = ast.get_arg(exp);
    assert!(ast.get(right).get_value() == "5");
    assert!(ast.get(ast.get_func(left)).get_value() == "add");
    assert!(ast.get(ast.get_arg(left)).get_value() == "2");

    Ok(())
}

#[test]
fn assign_2() -> Result<(), ParserError> {
    let str = "x = add (y z) 5";
    let mut parser = Parser::from_string(str.to_string());

    parser.bind("y".to_string());
    parser.bind("z".to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_assign_exp(assign);

    let left = ast.get_func(exp);
    let right = ast.get_arg(exp);
    assert!(ast.get(right).get_value() == "5");
    assert!(ast.get(ast.get_func(left)).get_value() == "add");

    let y_z = ast.get_arg(left);
    assert!(ast.get(ast.get_func(y_z)).get_value() == "y");
    assert!(ast.get(ast.get_arg(y_z)).get_value() == "z");

    println!("{}", ast.to_string(module));

    Ok(())
}

#[test]
fn multi_assign() -> Result<(), ParserError> {
    let str = "x = 5\n\n//Hello\ny = 6\nz = 7";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;

    let ass1 = ast.get_assign_to(module, "x".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass1)).get_value() == "5");

    let ass2 = ast.get_assign_to(module, "y".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass2)).get_value() == "6");

    let ass3 = ast.get_assign_to(module, "z".to_string()).unwrap();
    assert!(ast.get(ast.get_assign_exp(ass3)).get_value() == "7");

    assert!(ast.to_string(module) == "x = 5\ny = 6\nz = 7".to_string());

    Ok(())
}

#[test]
fn bound() -> Result<(), ParserError> {
    // recursive
    let str = "x = x 5";
    Parser::from_string(str.to_string()).parse_module()?;

    // add is an inbuilt
    let str = "x = add 2 x";
    Parser::from_string(str.to_string()).parse_module()?;

    // y is unbound
    let str = "x = add 2 y";
    Parser::from_string(str.to_string())
        .parse_module()
        .unwrap_err();

    Ok(())
}

#[test]
fn abstraction() -> Result<(), ParserError> {
    let str = "x = \\y :: Int. add y 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;

    assert_eq!(ast.to_string(module), "x = \\y :: Int . add y 5".to_string());

    // Should error because y is not bound
    let unbound_str = "x = (\\y :: Int . add y 5) y";
    let mut parser = Parser::from_string(unbound_str.to_string());
    assert!(parser.parse_module().is_err());

    // Should be same for both
    let multi_abstr = "x = \\y :: Int z :: Int . add y 5";
    let multi_abstr2 = "x = \\y :: Int . \\z :: Int . add y 5";
    let ast = Parser::from_string(multi_abstr.to_string()).parse_module()?;
    let ast2 = Parser::from_string(multi_abstr2.to_string()).parse_module()?;
    assert_eq!(ast.to_string(ast.root), ast2.to_string(ast2.root));

    let ignore_directive = "x = \\_ :: Int . 1.5";
    Parser::from_string(ignore_directive.to_string()).parse_module()?;

    Ok(())
}

#[test]
fn type_assignment() -> Result<(), ParserError> {
    let str = "x :: Int\nx = 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();

    let type_assignment = ast.get(assign).type_assignment.clone();
    assert!(type_assignment.is_some());
    assert!(type_assignment.unwrap().to_string() == "Int".to_string());

    Ok(())
}

#[test]
fn type_assignment_right_assoc() -> Result<(), ParserError> {
    let str = "x :: (Int -> Int) -> (Int -> Float) -> Int\nx = 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();

    let type_assignment = ast.get(assign).type_assignment.clone();
    assert!(type_assignment.is_some());
    assert_eq!(
        format!("{:?}", type_assignment.unwrap()),
        "(Int -> Int) -> ((Int -> Float) -> Int)"
    );

    Ok(())
}

#[test]
fn ite_1() -> Result<(), ParserError> {
    let str = "x = if true then 1 else 2";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;

    assert_eq!(ast.to_string(module), str);

    let str = "x = \\_ :: Int . add (if true then 1 else 2) (if true then 2 else 3)";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    assert_eq!(ast.to_string(module), str);

    Ok(())
}
