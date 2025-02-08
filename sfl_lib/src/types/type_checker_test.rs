use super::*;
use crate::{find_all_redex_contraction_pairs, Parser};
use crate::parser::ParserError;

fn tc_test_should_pass(program: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm).unwrap();
}

fn tc_test_should_fail(program: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm).unwrap_err();
}

#[test]
fn type_check_int_assign() {
    tc_test_should_pass("x :: Int\nx=10\nmain :: Int\nmain = x")
}

#[test]
fn type_check_int_add() {
    tc_test_should_pass("main :: Int\nmain = add 2 2")
}

#[test]
fn type_check_int_add_fail() {
    tc_test_should_fail("main :: Int -> Int\nmain = add 2 2")
}

#[test]
fn type_check_eq() {
    tc_test_should_pass("main :: Bool\nmain = eq (add 2 2) (mul 2 2)")
}

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
        "const_10 :: Float -> Int\nconst_10 = \\x . 10\nmain :: Int\nmain = const_10 2.0",
    );

    tc_test_should_fail(
        "const_10 :: Float -> Int\nconst_10 = \\x . 10\nmain :: Int\nmain = const_10 2",
    );
}

#[test]
fn type_check_abst() {
    tc_test_should_pass("main :: Int\nmain = (\\x y z.x) 10 10 10");
    tc_test_should_pass("main :: Int\nmain = (\\x y z a.x) 10 (\\x.x) 10 true");
    tc_test_should_pass("main :: Int -> Int\nmain = (\\x.x) (\\x.x)");
    tc_test_should_pass("main :: Int\nmain = (\\x.x) ((\\x.x) 10)");
    tc_test_should_fail("main :: Int\nmain = (\\x y. y) 10 true");
    tc_test_should_pass("main :: Int\nmain = (\\x y. x) 10 true");

    tc_test_should_pass("main :: Int -> Int\nmain = (\\x.x)");

    tc_test_should_pass("main :: Int -> Int\nmain = (\\x y. x) (\\x.x) (\\x.x)");
}

#[test]
fn type_y_combinator() {
    tc_test_should_fail("y :: Int -> Int -> Int\ny f = (\\x. f (x x)) (\\x. f (x x)) ");
}

#[test]
fn type_check_extra_arg_should_fail() {
    tc_test_should_fail(
        "const_10 :: Float -> Int\nconst_10 = \\x :: Float. 10\nmain :: Int\nmain = const_10 2.0 10",
    )
}

#[test]
fn type_check_const_abst() {
    tc_test_should_pass("main :: Float\nmain = (\\x y. x) 2.0 20");
    tc_test_should_pass("main :: Int\nmain = (\\_ . 10) 2.0");
    tc_test_should_fail("main :: Int\nmain = (\\x y . x) 2.0");
    tc_test_should_pass("main :: Int\nmain = (\\x y. y) 2.0 20");
    tc_test_should_pass("main :: Int\nmain = (\\x y. y) 2.0 20");
    tc_test_should_pass("main :: Int\nmain = (\\x :: Int -> Int. x) (\\x :: Int.x) 20");
}

#[test]
fn type_check_pair() {
    tc_test_should_pass("triple_first :: (a, (b, c)) -> a\ntriple_first (x, (y, z)) = x");
    tc_test_should_pass("triple_second :: (a, (b, c)) -> b\ntriple_second  (x, (y, z)) = y");
    tc_test_should_pass("triple_third :: (a, (b, c)) -> c\ntriple_third  (x, (y, z)) = z");
    inference_test("\\(x, (_, _)) . x", "∀a. ∀b. ∀c. (a, (b, c)) -> a");
    inference_test("\\(_, (x, _)) . x", "∀a. ∀b. ∀c. (a, (b, c)) -> b");
    inference_test("\\(_, (_, x)) . x", "∀a. ∀b. ∀c. (a, (b, c)) -> c");

    tc_test_should_pass("first :: (a, b) -> a\nfirst (x, y) = x");
    tc_test_should_pass("second :: (a, b) -> b\nsecond (x, y) = y");
    inference_test("\\(_, y) . y", "∀a. ∀b. (a, b) -> b");
    inference_test("\\(x, _) . x", "∀a. ∀b. (a, b) -> a");

    tc_test_should_pass("pair :: a -> b -> (a, b)\npair x y = (x, y)");
    inference_test("\\x y. (x, y)", "∀a. ∀b. a -> b -> (a, b)");
    inference_test(
        "\\x y z. (x, (y, z))",
        "∀a. ∀b. ∀c. a -> b -> c -> (a, (b, c))",
    );
    inference_test(
        "\\a b c d. ((a, b), (c, d))",
        "∀a. ∀b. ∀c. ∀d. a -> b -> c -> d -> ((a, b), (c, d))",
    );
}

fn inference_test(program: &str, type_str: &str) {
    let ast = Parser::from_string(program.to_string())
        .parse_tl_expression()
        .unwrap();
    let t = infer_type(&ast, ast.root).unwrap();
    assert_eq!(t.to_string(), type_str);
}

fn mod_main_inference_test(program: &str, type_str: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm).unwrap();
    let main_expr_type = ast
        .get(ast.get_main(ast.root).unwrap())
        .clone()
        .type_assignment
        .unwrap();
    assert_eq!(main_expr_type.to_string(), type_str);
}

fn mod_inference_should_fail(program: &str) {
    let pr = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm).unwrap_err();
}

fn expr_inference_should_fail(program: &str) {
    let mut ast = Parser::from_string(program.to_string())
        .parse_tl_expression()
        .unwrap();
    let expr = ast.root;
    infer_type(&mut ast, expr).unwrap_err();
}

#[test]
fn infer() {
    mod_main_inference_test("main f x = f x", "∀a. ∀b. (a -> b) -> a -> b");

    inference_test(
        "\\b . if true then (\\x . x) else (\\x . x)",
        "∀a. ∀b. a -> b -> b",
    );

    expr_inference_should_fail("\\x . x x");
    mod_inference_should_fail("recurse = recurse");

    inference_test("if true then (\\x :: Int. x) else (\\x . x)", "Int -> Int");
    inference_test(
        "\\b . if b then (\\x . x) else (\\x . 10)",
        "Bool -> Int -> Int",
    );
}

/// Test a program is well typed throughout evaluation
fn full_well_typed_test(program: &str) -> Result<(), TypeError> {
    let pr = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let mut main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
    let module = ast.root;
    infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm)?;
    let mut rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    while rcs.len() > 0 {
        #[cfg(debug_assertions)]
        let _module_str = ast.to_string_sugar(ast.root, true);

        let filtered_rcs = ast.filter_identical_rcs(&rcs);
        let laziest = ast.get_laziest_rc(main_expr, &filtered_rcs).unwrap();
        ast.do_rc_subst_and_identical_rcs(&laziest, &filtered_rcs);

        main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
        let module = ast.root;
        infer_or_check_assignment_types(&mut ast, module, &mut lt, &tm)?;
        rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    }
    Ok(())
}

#[test]
fn full_well_typed_tests() -> Result<(), TypeError> {
    let program = r#"
    fac :: Int -> Int
    fac n = if n <= 1 then 1 else n * (fac (n - 1))

    main :: Int
    main = fac 5"#;
    full_well_typed_test(program)?;

    Ok(())
}

#[test]
fn alias_test() -> Result<(), TypeError> {
    tc_test_should_pass("type IntAlias = Int\nmain :: IntAlias -> Int\nmain = \\x.x");
    tc_test_should_fail("type IntAlias = Bool\nmain :: IntAlias -> Int\nmain = \\x.x");
    tc_test_should_fail("type IntAlias = Int\nmain :: IntAlias -> Bool\nmain = \\x.x");

    Ok(())
}

#[test]
fn maybe_test() -> Result<(), TypeError> {
    tc_test_should_pass("data Maybe a = Some a | None\nmain :: a -> Maybe a\nmain = \\x. Some x");
    tc_test_should_fail("data Maybe a = Some a | None\nmain :: a -> Int\nmain = \\x. Some x");

    Ok(())
}

#[test]
fn either_test() -> Result<(), TypeError> {
    tc_test_should_fail(
        "data Either a b = Left a | Right b\nmain :: a -> Either a b\nmain = \\x. Right x",
    );
    tc_test_should_pass(
        "data Either a b = Left a | Right b\nmain :: a -> Either a b\nmain = \\x. Left x",
    );

    mod_main_inference_test(
        "data Either a b = Left a | Right b\nmain = \\x. Left x",
        "∀a. ∀b. a -> Either a b",
    );
    mod_main_inference_test(
        "data Either a b = Left a | Right b\nmain = \\x. Right x",
        "∀a. ∀b. a -> Either b a",
    );

    Ok(())
}

#[test]
fn list_text() -> Result<(), TypeError> {
    mod_main_inference_test(
        "data List a = Cons a (List a) | Nil\ndata IntListEither a = Left (List Int) | Right a\nmain = \\x.Left (Cons x Nil)",
        "∀a. Int -> IntListEither a"
    );
    mod_main_inference_test(
        "data List a = Cons a (List a) | Nil\ndata IntListEither a = Left (List Int) | Right a\nmain = \\x.Left (Cons x (Cons 10 Nil))",
        "∀a. Int -> IntListEither a"
    );

    tc_test_should_pass("data List a = Cons a (List a) | Nil\ndata IntListEither a = Left (List Int) | Right a\nmain :: Int -> (IntListEither a)\nmain = \\x.Left (Cons x (Cons 10 Nil))");

    Ok(())
}

#[test]
fn triple_test() -> Result<(), TypeError> {
    mod_main_inference_test(
        "data Triple a b c = Triple a b c | NoTriple\nmain = \\x y z. Triple x y z",
        "∀a. ∀b. ∀c. a -> b -> c -> Triple a b c",
    );

    mod_main_inference_test(
        "data Triple a b c = Triple a b c | NoTriple\nmain = NoTriple",
        "∀a. ∀b. ∀c. Triple a b c",
    );

    Ok(())
}

#[test]
fn check_match_length() -> Result<(), ParserError> {
    let program = r#"
    data List a = Cons a (List a) | Nil

    length x = match x {
       | Nil       -> 0
       | Cons _ xs -> 1 + length xs
    }

    main = length (Cons 1 (Cons 2 (Cons 3 Nil)))"#;

    mod_main_inference_test(program, "Int");
    
    Ok(())
}