use crate::functions::LabelTable;

use super::*;

/// This will need to be significantly changed when types introduced
/// This will check for applications to inbuilts with the right num
/// of chars. For example, a call to a inbuilt add could be:
/// add 2 3
/// Which would look like
/// App[[App add 2], 3]
/// This function checks that the rhs is a literal, and the lhs is
/// either an ID or an App of an ID in the set of inbuilts and a literal
fn check_for_ready_call_to_inbuilts(
    ast: &AST,
    exp: usize,
    inbuilts: &LabelTable,
) -> Option<AST> {
    let mut f = ast.get_func(exp);
    let mut x = ast.get_arg(exp);
    let mut argv = vec![];

    // can add type assertion here that there exists B and A s.t. x :: B and f :: B -> A
    for _ in 1..inbuilts.get_max_arity() {
        match ast.get(x).t {
            ASTNodeType::Literal => {
                argv.push(ast.get(x));
            }
            _ => return None,
        }

        match ast.get(f).t {
            ASTNodeType::Identifier => {
                let inbuilts_of_arity = inbuilts.get_n_ary_inbuilts(argv.len());
                let val = ast.get(f).get_value();
                if inbuilts_of_arity.contains_key(&val) {
                    return Some(inbuilts_of_arity.get(&val).unwrap().call_inbuilt(ast.get(f), argv));
                } else {
                    return None;
                }
            }
            ASTNodeType::Application => {
                x = ast.get_arg(f);
                f = ast.get_func(f);
            }
            _ => return None,
        }
    }

    None
}

pub fn find_redex_contraction_pairs(ast: &AST, module: usize, exp: usize, lt: &LabelTable) -> Vec<(usize, AST)> {
    vec![]
}
