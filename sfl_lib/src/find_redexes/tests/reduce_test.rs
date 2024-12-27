use crate::{find_redexes::reduce::*, ASTNodeType, Parser};

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
    let redex = ast.get(rc.0);
    let contraction = rc.1.get(rc.1.root);
    assert_eq!(format!("{} => {:?}", ast.to_string(rc.0), rc.1), "add 5 1 => 6")
}
