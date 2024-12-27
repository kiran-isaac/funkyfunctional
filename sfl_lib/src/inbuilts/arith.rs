use parser::{Token, TokenType};

use super::assert_prim_type;
use crate::*;
use std::{fmt::Display, ops::{Add, Div, Mul, Sub}, str::FromStr};

fn inbuilt_binary<T>(call: &ASTNode, args: Vec<&ASTNode>, op: fn(T, T) -> T, p : Primitive) -> ASTNode 
where
    T: FromStr + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Display,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let a = args[0];
    let b = args[1];

    assert_prim_type(&a.get_lit_type(), p);
    assert_prim_type(&b.get_lit_type(), p);

    let a_int: T = a.get_value().parse().unwrap();
    let b_int: T = b.get_value().parse().unwrap();

    let c_int = op(b_int, a_int);

    ASTNode::new_lit(
        Token {
            tt: TokenType::IntLit,
            value: format!("{}", c_int),
        },
        call.line,
        call.col,
    )
}

pub fn inbuilt_int_add(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, i64::add, Primitive::Int64)
}

pub fn inbuilt_int_sub(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, i64::sub, Primitive::Int64)
}

pub fn inbuilt_int_mul(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, i64::mul, Primitive::Int64)
}

pub fn inbuilt_int_div(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, i64::div, Primitive::Int64)
}

pub fn inbuilt_float_add(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, f64::add, Primitive::Float64)
}

pub fn inbuilt_float_sub(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, f64::sub, Primitive::Float64)
}

pub fn inbuilt_float_mul(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, f64::mul, Primitive::Float64)
}

pub fn inbuilt_float_div(call: &ASTNode, args: Vec<&ASTNode>) -> ASTNode {
    inbuilt_binary(call, args, f64::div, Primitive::Float64)
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
