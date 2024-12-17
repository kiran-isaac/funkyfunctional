use crate::{ASTNode, ASTNodeType, Parser};
use crate::find_redexes;

#[test]
fn test_find_redexes() {
    let str = "x = 1\ny = x".to_string();
    let ast = Parser::from_string(str).parse().unwrap();

    let redexes = find_redexes(&ast);

    println!("{:?}", redexes);
}