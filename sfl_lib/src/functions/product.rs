use crate::*;

pub fn pair_first(_: &ASTNode, args: Vec<&ASTNode>) -> AST {
    assert_eq!(args.len(), 1);
    let pair = args[0];
    assert_eq!(pair.t == ASTNodeType::Pair);
    assert_eq!(pair.children.len(), 2);
    AST::clone_node(pair.children[0])
}