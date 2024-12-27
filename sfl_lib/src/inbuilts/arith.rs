use parser::{Token, TokenType};

use super::assert_prim_type;
use crate::*;
use std::ops::{Add, Div, Mul, Sub};

fn inbuilt_int_binary(call: &ASTNode, args: Vec<&ASTNode>, op: fn(i64, i64) -> i64) -> ASTNode {
    let a = args[0];
    let b = args[1];

    assert_prim_type(&a.get_lit_type(), Primitive::Int64);
    assert_prim_type(&b.get_lit_type(), Primitive::Int64);

    let a_int: i64 = a.get_value().parse().unwrap();
    let b_int: i64 = b.get_value().parse().unwrap();

    let c_int = op(a_int, b_int);

    ASTNode::new_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", c_int),
        },
        a.line,
        a.col,
    )
}

pub fn inbuilt_int_add(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_int_binary(call, args, i64::add)
}

pub fn inbuilt_int_sub(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_int_binary(call, args, i64::sub)
}

pub fn inbuilt_int_mul(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_int_binary(call, args, i64::mul)
}

pub fn inbuilt_int_div(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_int_binary(call, args, i64::div)
}

pub fn inbuilt_int_zero(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    assert!(args.len() == 0);

    ASTNode::new_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", 0),
        },
        call.line,
        call.col,
    )
}
