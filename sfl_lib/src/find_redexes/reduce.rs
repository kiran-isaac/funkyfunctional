use crate::inbuilts::InbuiltsLookupTable;

use super::*;

struct InbuiltReduction {
    name : String,
    arity : usize,
    args : Vec<usize>,
}

/// This will need to be significantly changed when types introduced
/// This will check for applications to inbuilts with the right num
/// of chars. For example, a call to a inbuilt add could be:
/// add 2 3
/// Which would look like
/// App[[App add 2], 3]
/// This function checks that the rhs is a literal, and the lhs is
/// either an ID or an App of an ID in the set of inbuilts and a literal
fn check_for_correct_call_to_inbuilts(ast : &AST, module : usize, exp : usize, inbuilts : &InbuiltsLookupTable) -> Option<InbuiltReduction> {
    None
}

fn find_redex_contraction_pairs(ast : &AST, module : usize, exp : usize) -> Vec<(usize, AST)> {
    let mut pairs : Vec<(usize, AST)> = vec![];
    let previous_assignments = ast.get_assigns_map(module);
    let inbuilts = InbuiltsLookupTable::new();

    match ast.get(exp).t {
        ASTNodeType::Application => {
            let inbuilt_check_result = check_for_correct_call_to_inbuilts(ast, module, exp, &inbuilts);
        }
        _ => unimplemented!()
    }

    pairs
}