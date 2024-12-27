use crate::*;
use super::{assert_prim_type, InbuiltFuncCallResult};
use std::ops::{Add, Sub, Mul, Div};

fn inbuilt_int_binary(args : Vec<&ASTNode>, op : fn(i64, i64) -> i64) -> InbuiltFuncCallResult {
    let a = args[0];
    let b = args[1];

    assert_prim_type(&a.get_lit_type(), Primitive::Int64);
    assert_prim_type(&b.get_lit_type(), Primitive::Int64);

    let a_int : i64 = a.get_value().parse().unwrap();
    let b_int : i64 = b.get_value().parse().unwrap();

    let c_int = op(a_int, b_int);

    (Primitive::Int64, format!("{}", c_int))
}

pub fn inbuilt_int_add(args : Vec<&ASTNode>) -> InbuiltFuncCallResult {
    inbuilt_int_binary(args, i64::add)
}   

pub fn inbuilt_int_sub(args : Vec<&ASTNode>) -> InbuiltFuncCallResult {
    inbuilt_int_binary(args, i64::sub)

}   

pub fn inbuilt_int_mul(args : Vec<&ASTNode>) -> InbuiltFuncCallResult {
    inbuilt_int_binary(args, i64::mul)
}   

pub fn inbuilt_int_div(args : Vec<&ASTNode>) -> InbuiltFuncCallResult {
    inbuilt_int_binary(args, i64::div)
}   