use crate::*;

pub fn inbuilt_const1(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 2);
    AST::single_node(args[0].clone())
}

pub fn inbuilt_const2(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 2);
    AST::single_node(args[1].clone())
}

pub fn inbuilt_id(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    AST::single_node(args[0].clone())
}

pub fn inbuilt_if(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].get_lit_type(), Type::Primitive(Primitive::Bool));
    if args[0].get_value() == "true" {
        Parser::from_string("\\x y. x".to_string())
            .parse_tl_expression()
            .unwrap()
    } else {
        Parser::from_string("\\x y. y".to_string())
            .parse_tl_expression()
            .unwrap()
    }
}
