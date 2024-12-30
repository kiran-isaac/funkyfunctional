use crate::*;

pub fn inbuilt_const1(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 2);
    AST::single_node(args[1].clone())
}

pub fn inbuilt_const2(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 2);
    AST::single_node(args[0].clone())
}

pub fn inbuilt_id(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 1);
    AST::single_node(args[0].clone())
}

pub fn inbuilt_if(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 1);
    assert!(args[0].get_lit_type() == Type::Primitive(Primitive::Bool));
    if args[0].get_value() == "true" {
        Parser::from_string("const1".to_string())
            .parse_tl_expression()
            .unwrap()
    } else {
        Parser::from_string("const2".to_string())
            .parse_tl_expression()
            .unwrap()
    }
}
