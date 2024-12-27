use parser::Token;

use super::super::*;
use crate::{AST, ASTNodeType};

#[test]
fn test_basic_arith() {
    let mut ast = AST::new();
    let inbuilts = InbuiltsLookupTable::new();
    let add = inbuilts.get(2, "add".to_string()).unwrap();
    let sub = inbuilts.get(2, "sub".to_string()).unwrap();
    let mul = inbuilts.get(2, "mul".to_string()).unwrap();
    let div = inbuilts.get(2, "div".to_string()).unwrap();

    for _ in 0..1000 {
        // Generate random numbers, (16 bit to avoid overflow)
        let a_int = rand::random::<i16>();
        let b_int = rand::random::<i16>();

        let a = Token{tt: parser::TokenType::IntLit, value: format!("{}", a_int)};
        let b = Token{tt: parser::TokenType::IntLit, value: format!("{}", b_int)};

        let a_int = a_int as i64;
        let b_int = b_int as i64;

        let a = ast.add_lit(a, 0, 0);
        let b = ast.add_lit(b, 0, 0);

        let c_add = add.call(vec![ast.get(a), ast.get(b)]);
        let c_sub = sub.call(vec![ast.get(a), ast.get(b)]);
        let c_mul = mul.call(vec![ast.get(a), ast.get(b)]);
        let c_div = div.call(vec![ast.get(a), ast.get(b)]);

        assert!(c_add.0 == Primitive::Int64);
        assert!(c_sub.0 == Primitive::Int64);
        assert!(c_mul.0 == Primitive::Int64);
        assert!(c_div.0 == Primitive::Int64);
        
        assert_eq!(c_add.1.parse::<i64>().unwrap(), a_int + b_int);
        assert_eq!(c_sub.1.parse::<i64>().unwrap(), a_int - b_int);
        assert_eq!(c_mul.1.parse::<i64>().unwrap(), a_int * b_int);
        assert_eq!(c_div.1.parse::<i64>().unwrap(), a_int / b_int);

    }
}