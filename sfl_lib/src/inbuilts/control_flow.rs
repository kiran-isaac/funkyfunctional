use parser::{Token, TokenType};
use crate::*;

pub fn inbuilt_const(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 2);
    AST::single_node(args[1].clone())
}

pub fn inbuilt_unconst(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 2);
    AST::single_node(args[0].clone())
}

pub fn inbuilt_if(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 1);
    assert!(args[0].get_lit_type() == Type::Primitive(Primitive::Bool));
    let mut ast = AST::new();

    ast.add_id(
        Token {
            tt: TokenType::Id,
            value: (if args[0].get_value() == "true" {
                "const"
            } else {
                "unconst"
            })
            .to_string(),
        },
        0,
        0,
    );

    ast
}