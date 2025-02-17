use super::*;

fn full_run_test(program: String) -> String {
    let pr = Parser::from_string(program).parse_module(true).unwrap();
    let mut ast = pr.ast;
    let mut lt = pr.lt;
    let tm = pr.tm;
    let module = ast.root;
    typecheck(&mut ast, module, &mut lt, &tm).unwrap();
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
