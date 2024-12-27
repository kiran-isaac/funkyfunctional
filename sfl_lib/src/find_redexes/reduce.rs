use crate::inbuilts::InbuiltsLookupTable;

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
    module: usize,
    exp: usize,
    inbuilts: &InbuiltsLookupTable,
) -> Option<ASTNode> {
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
                    return Some(inbuilts_of_arity.get(&val).unwrap().call(ast.get(f), argv));
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

pub fn find_redex_contraction_pairs(ast: &AST, module: usize, exp: usize) -> Vec<(usize, AST)> {
    let mut pairs: Vec<(usize, AST)> = vec![];
    
    // Dont need to worry about this as main must be at the end, so everything defined in 
    // the module is defined here
    let previous_assignments = ast.get_assigns_map(module);
    let inbuilts = InbuiltsLookupTable::new();

    match ast.get(exp).t {
        ASTNodeType::Identifier => {
            let value = ast.get(exp).get_value();

            // It should not be non zero_ary func as otherwise it would be caught by the app case
            if inbuilts.get_n_ary_inbuilts(0).contains_key(&value) {
                let inbuilt = inbuilts.get_n_ary_inbuilts(0).get(&value).unwrap();
                let result = inbuilt.call(&ast.get(exp), vec![]);

                let mut res_ast = AST::new();
                let res_i = res_ast.add(result);
                res_ast.root = res_i;

                pairs.push((exp, res_ast));
            } else if previous_assignments.contains_key(&value) {
                let n = ast.get(ast.get_exp(*previous_assignments.get(&value).unwrap()));
                pairs.push((exp, AST::single_node(n.clone())));
            }
        }
        ASTNodeType::Application => {
            if let Some(inbuilt_reduction) =
                check_for_ready_call_to_inbuilts(ast, module, exp, &inbuilts)
            {
                pairs.push((exp, AST::single_node(inbuilt_reduction)));
            } else {
                let f = ast.get_func(exp);
                let x = ast.get_arg(exp);
                match ast.get(f).t {
                    ASTNodeType::Application => {
                        pairs.extend(find_redex_contraction_pairs(ast, module, f));
                    }
                    ASTNodeType::Abstraction => {
                        // All usages of the abstracted variable
                        let var_name = ast.get(ast.get_abstr_var(f)).get_value();
                        let mut cloned_abst_expr = ast.clone_node(ast.get_abstr_exp(f));

                        let usages = cloned_abst_expr.get_all_instances_of_var_in_exp(cloned_abst_expr.root, &var_name);
                        let arg_id = cloned_abst_expr.append(&ast, x);

                        for usage in usages {
                            cloned_abst_expr.replace(usage, arg_id);
                        }

                        pairs.push((exp, cloned_abst_expr))
                    }
                    _ => unimplemented!()
                }

                pairs.extend(find_redex_contraction_pairs(ast, module, ast.get_arg(exp)));
            }
        }
        _ => {}
    }

    pairs
}
