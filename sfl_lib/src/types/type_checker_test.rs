use super::*;
use crate::Parser;

fn tc_test_should_pass(program: &str) {
    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut tc = TypeChecker::new();
    tc.check_module(&ast, ast.root).unwrap();
}

fn tc_test_should_fail(program: &str) {
    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut tc = TypeChecker::new();
    tc.check_module(&ast, ast.root).unwrap_err();
}

// #[test]
// fn type_check_int_assign() {
//     tc_test_should_pass("x :: Int\nx=10\nmain :: Int\nmain = x")
// }

// #[test]
// fn type_check_int_add() {
//     tc_test_should_pass("main :: Int\nmain = add 2 2")
// }

// #[test]
// fn type_check_int_add_fail() {
//     tc_test_should_fail("main :: Int -> Int\nmain = add 2 2")
// }

// #[test]
// fn type_check_eq() {
//     tc_test_should_pass("main :: Bool\nmain = eq (add 2 2) (mul 2 2)")
// }

#[test]
fn type_check_ite() {
    tc_test_should_pass("main :: Float\nmain = if false then 2.0 else 3.0");
    tc_test_should_pass("main :: Int\nmain = if false then 2 else 3");
    tc_test_should_pass("main :: Bool\nmain = if true then true else false");

    tc_test_should_fail("main :: Int\nmain = if false then 2.0 else 3");
    tc_test_should_fail("main :: Float\nmain = if false then 2.0 else true");
}

#[test]
fn type_check_const_int_abst() {
    tc_test_should_pass(
        "const_10 :: Float -> Int\nconst_10 = \\x :: Float. 10\nmain :: Int\nmain = const_10 2.0",
    )
}

#[test]
fn type_check_abst() {
    tc_test_should_pass("main :: Int -> Int\nmain = (\\x y. x) (\\x :: Int.x) (\\x :: Int.x)");

    // Should not need type annotation
    tc_test_should_pass("main :: Int -> Int\nmain = (\\x.x)");
    tc_test_should_pass("main :: Int -> Int\nmain = (\\x::Int.x)");
    tc_test_should_fail("main :: Int -> Int\nmain = (\\x::Bool.x)");
    tc_test_should_fail("main :: Int -> Int\nmain = (\\x::Float.x)");
}

#[test]
fn type_check_extra_arg_should_fail() {
    tc_test_should_fail(
        "const_10 :: Float -> Int\nconst_10 = \\x :: Float. 10\nmain :: Int\nmain = const_10 2.0 10",
    )
}

#[test]
fn type_check_const_abst() {
    tc_test_should_pass("main :: Float\nmain = (\\x :: Float y :: Int . x) 2.0 20");
    // tc_test_should_pass("main :: Int\nmain = (\\_ :: Float . 10) 2.0");
    // tc_test_should_fail("main :: Int\nmain = (\\x :: Float y :: Int . x) 2.0");
    // tc_test_should_pass("main :: Int\nmain = (\\x :: Float y :: Int . y) 2.0 20");
    // tc_test_should_pass("main :: Int\nmain = (\\x :: Int -> Int. x) (\\x :: Int.x) 20");
}

// #[test]
// fn type_check_extra_arg_should_fail() {
//     let program =
//         "const_10 :: Float -> Int\nconst_10 = \\x. 10\nmain :: Int\nmain = const_10 2.0 10";

//     let ast = Parser::from_string(program.to_string())
//         .parse_module()
//         .unwrap();
//     let mut tc = TypeChecker::new();
//     println!("{:?}", tc.check_module(&ast, ast.root).unwrap_err());
// }
