use crate::find_redexes::get_replacements;
use crate::find_redexes::replace::do_replacement;
use crate::{ASTNode, ASTNodeType, Parser};

#[test]
fn test_find_replacements() {
    let assigns = "x = 1\ny = x".to_string();
    let exp = "add x 3".to_string();
    let mut mod_parser = Parser::from_string(assigns);
    let mod_ast = mod_parser.parse_module().unwrap();

    let mut expr_parser = Parser::from_string(exp);
    expr_parser.add_bindings_from(&mod_parser);
    let expr_ast = expr_parser.parse_tl_expression().unwrap();

    let replacements = get_replacements(&expr_ast, &mod_ast);
    assert!(replacements.len() == 1);

    assert_eq!(expr_ast.get(replacements[0].0).get_value(), "x");
    assert_eq!(mod_ast.get(replacements[0].1).get_value(), "1");
}

#[test]

fn test_do_replacement() {
    let assigns = "x = 1\ny = x".to_string();
    let exp = "add 1 y".to_string();
    let mut mod_parser = Parser::from_string(assigns);
    let mod_ast = mod_parser.parse_module().unwrap();

    let mut expr_parser = Parser::from_string(exp);
    expr_parser.add_bindings_from(&mod_parser);
    let mut expr_ast = expr_parser.parse_tl_expression().unwrap();

    // Do replacement once, should switch y for x
    let replacements = get_replacements(&expr_ast, &mod_ast);
    assert!(replacements.len() == 1);
    do_replacement(&mod_ast, &mut expr_ast, replacements[0]);
    assert_eq!(expr_ast.to_string(expr_ast.root), "add 1 x");

    // Do replacement again, should switch x for 1
    let replacements = get_replacements(&expr_ast, &mod_ast);
    assert!(replacements.len() == 1);
    do_replacement(&mod_ast, &mut expr_ast, replacements[0]);
    assert_eq!(expr_ast.to_string(expr_ast.root), "add 1 1");
}