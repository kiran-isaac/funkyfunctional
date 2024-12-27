use crate::{find_redexes::reduce::*, parser::ParserError, ASTNodeType, Parser, AST};

/// O(n^2) so only use for small things
fn assert_eq_in_any_order<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) {
    for x in a {
        let mut found = false;
        for y in b {
            if x == y {
                found = true;
            }
        }
        assert!(found);
    }
}

fn rc_pair_to_string(ast: &AST, rc: &(usize, AST)) -> String {
    format!("{} => {:?}", ast.to_string(rc.0), rc.1)
}

#[test]
fn zero_test() {
    let program = "main = zero";
    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();

    let module = ast.root;
    let exp = ast.get_exp(ast.get_main(module).unwrap());

    let rcs = find_redex_contraction_pairs(&ast, module, exp);
    assert!(rcs.len() == 1);

    let rc = &rcs[0];
    let redex = ast.get(rc.0);
    let result = rc.1.get(rc.1.root);

    assert!(redex.t == ASTNodeType::Identifier);
    assert!(redex.get_value() == "zero");
    assert!(result.t == ASTNodeType::Literal);
    assert!(result.get_value() == "0");
}

#[test]
fn basic_add_test() {
    let program = "main = add 5 1";
    let ast = Parser::from_string(program.to_string())
        .parse_module()
        .unwrap();

    let module = ast.root;
    let exp = ast.get_exp(ast.get_main(module).unwrap());

    let rcs = find_redex_contraction_pairs(&ast, module, exp);
    assert!(rcs.len() == 1);

    let rc = &rcs[0];
    assert_eq!(rc_pair_to_string(&ast, rc), "add 5 1 => 6")
}

#[test]
fn multi_op_test() {
    let a_int = rand::random::<u16>() as i64;
    let b_int = rand::random::<u16>() as i64;
    let c_int = rand::random::<u16>() as i64;
    let d_int = rand::random::<u16>() as i64;
    let program = format!(
        "main = sub (add {} {}) (mul {} {})",
        a_int, b_int, c_int, d_int
    );
    let mut ast = Parser::from_string(program).parse_module().unwrap();

    let module = ast.root;
    let exp = ast.get_exp(ast.get_main(module).unwrap());

    let rcs = find_redex_contraction_pairs(&ast, module, exp);

    let correct = vec![
        format!("add {} {} => {}", a_int, b_int, a_int + b_int),
        format!("mul {} {} => {}", c_int, d_int, c_int * d_int),
    ];

    let proposed: Vec<String> = rcs.clone()
        .into_iter()
        .map(|rc| rc_pair_to_string(&ast, &rc))
        .collect();

    assert_eq_in_any_order(&correct, &proposed);

    for (old, new) in rcs {
        ast.replace_from_other_root(&new, old);
    }

    let rcs = find_redex_contraction_pairs(&ast, module, exp);
    assert!(rcs.len() == 1);
    for (old, new) in rcs {
        ast.replace_from_other_root(&new, old);
    }

    assert_eq!(format!("main = {}", (a_int + b_int) - (c_int * d_int)), format!{"{:?}", ast})

}

