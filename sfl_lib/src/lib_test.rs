use super::*;

fn full_run_test(program: &str, typechecked: bool) -> String {
    let pr = Parser::from_string(program.to_string()).parse_module(true).unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    if typechecked {
        typecheck(&mut ast, module, &mut lt, &tm).unwrap();
    }
    let mut main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());

    let mut rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    while rcs.len() != 0 {
        let rc = &rcs[0];
        ast.do_rc_subst(main_expr, rc);

        main_expr = ast.get_assign_exp(ast.get_main(ast.root).unwrap());
        rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    }
    ast.to_string_sugar(main_expr, false)
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
    "#;

    assert_eq!(full_run_test(program, true), "1");
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
    "#;

    assert_eq!(full_run_test(program, true), "2.5");
}

#[test]
fn untyped_y_combinator() {
    let program = r#"
    fac f n = if (n <= 1) (1) (n * (f (n - 1)))
    y f = (\x. f (x x)) (\x. f (x x))

    main = y fac 5"#;

    full_run_test(program, false);
}
