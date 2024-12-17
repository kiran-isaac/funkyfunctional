use crate::parser::*;

#[test]
fn assign() -> Result<(), ParserError> {
    let str = "x = add 2 5";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse()?;
    let assign = ast.get_assign_to("x".to_string()).unwrap();

    assert!(assign.get_assignee().get_value() == "x");
    assert!(assign.get_exp().get_func().get_func().get_value() == "add");
    assert!(assign.get_exp().get_func().get_arg().get_value() == "2");
    assert!(assign.get_exp().get_arg().get_value() == "5");
    
    Ok(())
}

#[test]
fn assign_2() -> Result<(), ParserError> {
    let str = "x = add (y z) 5";
    let mut parser = Parser::from_string(str.to_string());
    
    parser.bind("y".to_string());
    parser.bind("z".to_string());

    let ast = parser.parse()?;
    let assign = ast.get_assign_to("x".to_string()).unwrap();

    assert!(assign.get_assignee().get_value() == "x");
    assert!(assign.get_exp().get_func().get_func().get_value() == "add");
    assert!(assign.get_exp().get_func().get_arg().get_func().get_value() == "y");
    assert!(assign.get_exp().get_func().get_arg().get_arg().get_value() == "z");
    assert!(assign.get_exp().get_arg().get_value() == "5");
    
    Ok(())
}

#[test]
fn multi_assign() -> Result<(), ParserError> {
    let str = "x = 5\n\n//Hello\ny = 6\nz = 7";
    let mut parser = Parser::from_string(str.to_string());

    let ast = parser.parse()?;
    let assign = ast.get_assign_to("x".to_string()).unwrap();

    assert!(assign.get_assignee().get_value() == "x");
    assert!(assign.get_exp().get_value() == "5");

    let assign = ast.get_assign_to("y".to_string()).unwrap();

    assert!(assign.get_assignee().get_value() == "y");
    assert!(assign.get_exp().get_value() == "6");

    let assign = ast.get_assign_to("z".to_string()).unwrap();

    assert!(assign.get_assignee().get_value() == "z");
    assert!(assign.get_exp().get_value() == "7");
    
    Ok(())
}