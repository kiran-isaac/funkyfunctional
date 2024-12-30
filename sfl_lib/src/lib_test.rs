use super::*;

fn full_run_test(program: String) -> String {
    let mut ast = Parser::from_string(program).parse_module().unwrap();
    let mut tc = TypeChecker::new();
    let lt = tc.check_module(&ast, ast.root).unwrap();
    let mut exp = ast.get_assign_exp(ast.get_main(ast.root));

    let mut rcs = find_redex_contraction_pairs(&ast, ast.root, exp, lt);
    while rcs.len() != 0 {
        let rc = &rcs[0];
        ast.do_rc_subst(rc);

        exp = ast.get_assign_exp(ast.get_main(ast.root));
        rcs = find_redex_contraction_pairs(&ast, ast.root, exp, lt);
    }
    ast.to_string(exp)
}

#[test]
fn full_run_1() {
    let program = r#"
    x :: Int 
    x = 5

    y :: Int
    y = 2

    inc :: Int -> Int
    inc = \i :: Int . add i 1

    main :: Int
    main = sub (add 5 (inc x)) (mul 5 y)
    "#
    .to_string();

    assert_eq!(full_run_test(program), "1");
}

#[test]
fn full_run_2() {
    let program = r#"
    x :: Int
    x = 100

    const_float::Int -> Float
    const_float = \_ :: Int. 1.5

    y :: Float
    y = const_float x

    inc :: Float -> Float
    inc = \i :: Float. addf i 1.0

    main :: Float
    main = inc y
    "#
    .to_string();

    assert_eq!(full_run_test(program), "2.5");
}
