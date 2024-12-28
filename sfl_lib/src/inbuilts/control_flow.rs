use parser::{Token, TokenType};

use super::assert_prim_type;
use crate::*;

// pub fn inbuilt_int_const(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
//     assert!(args.len() == 2);
//     AST::single_node(args[0].clone())
// }

// pub fn inbuilt_int_unconst(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
//     assert!(args.len() == 2);
//     AST::single_node(args[1].clone())
// }

pub fn inbuilt_int_if(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert!(args.len() == 1);
    assert!(args[0].get_lit_type() == Type::Primitive(Primitive::Bool));
    if args[0].get_value() == "true" {
        Parser::from_string("\\x _.x".to_string())
            .parse_tl_expression()
            .unwrap()
    } else {
        Parser::from_string("\\_ x.x".to_string())
            .parse_tl_expression()
            .unwrap()
    }
}
