use crate::{parser::*, Primitive, Type, TypeError};

#[test]
fn type_literals() -> Result<(), TypeError> {
    let str = "true";
    let mut p = Parser::from_string(str.to_string());
    let ast = p.parse_tl_expression().unwrap();
    assert!(ast.get_type(0)? == Type::Primitive(Primitive::Bool));

    let str = "false";
    let mut p = Parser::from_string(str.to_string());
    let ast = p.parse_tl_expression().unwrap();
    assert!(ast.get_type(0)? == Type::Primitive(Primitive::Bool));

    let str = "1";
    let mut p = Parser::from_string(str.to_string());
    let ast = p.parse_tl_expression().unwrap();
    assert!(ast.get_type(0)? == Type::Primitive(Primitive::Int64));

    let str = "1.0";
    let mut p = Parser::from_string(str.to_string());
    let ast = p.parse_tl_expression().unwrap();
    assert!(ast.get_type(0)? == Type::Primitive(Primitive::Float64));

    let str = "'a'";
    let mut p = Parser::from_string(str.to_string());
    let ast = p.parse_tl_expression().unwrap();
    assert!(ast.get_type(0)? == Type::Primitive(Primitive::Char));

    Ok(())
}