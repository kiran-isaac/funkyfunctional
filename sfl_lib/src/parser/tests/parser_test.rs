use crate::parser::*;

#[test]
fn assign() -> Result<(), ParserError> {
    let str = "x = add 2 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse_module()?;
    let module = 0;
    let assign = ast.get_assign_to(module, "x".to_string()).unwrap();
    let exp = ast.get_exp(assign);

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
    let exp = ast.get_exp(assign);

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
    assert!(ast.get(ast.get_exp(ass1)).get_value() == "5");

    let ass2 = ast.get_assign_to(module, "y".to_string()).unwrap();
    assert!(ast.get(ast.get_exp(ass2)).get_value() == "6");

    let ass3 = ast.get_assign_to(module, "z".to_string()).unwrap();
    assert!(ast.get(ast.get_exp(ass3)).get_value() == "7");

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
    Parser::from_string(str.to_string()).parse_module().unwrap_err();

    Ok(())
}