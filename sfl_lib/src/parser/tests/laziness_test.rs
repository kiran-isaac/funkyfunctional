use crate::{find_all_redex_contraction_pairs, KnownTypeLabelTable, Parser};

#[test]
fn test_laziness() {
    let program = "(\\x y. x) ((\\x. 1) true) ((\\x. x + 1) 2)".to_string();
    let ast = Parser::from_string(program).parse_tl_expression().unwrap();
    let rcs = find_all_redex_contraction_pairs(&ast, None, ast.root, &KnownTypeLabelTable::new());
    let rc = ast.get_laziest_rc(ast.root, &rcs).unwrap();
    let s2 = rc.1.to_string_sugar(rc.1.root, false);
    assert_eq!(s2, "(\\x. 1) true");
}
