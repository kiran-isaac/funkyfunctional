use crate::functions::LabelTable;
use std::collections::HashMap;

use super::*;

/// This will check for applications to functions:
/// - lables with func types
/// - lambda abstractions
/// - inbuilt functions
/// with the right num of args.
/// For example, a call to a inbuilt add could be: add 2 3
/// Which would look like
/// App[[App add 2], 3]
/// This function checks that the rhs is a literal, and the lhs is
/// either a function or an App of a function in the set of funcs
/// with the right num of args
fn check_for_ready_call(
    ast: &AST,
    exp: usize,
    lt: &LabelTable,
    am: HashMap<String, usize>,
) -> Option<AST> {
    let mut f = ast.get_func(exp);
    let mut x = ast.get_arg(exp);
    let mut argv = vec![];
    let mut argv_ids = vec![];

    // True if only literals encountered. If true, then we can call inbuilt functions
    let mut literals_only = true;

    for _ in 1..lt.get_max_arity() {
        argv.push(ast.get(x));
        argv_ids.push(x);
        match ast.get(x).t {
            ASTNodeType::Literal => {}
            _ => literals_only = false,
        }

        match ast.get(f).t {
            ASTNodeType::Identifier => {
                let labels_of_arity = lt.get_n_ary_labels(argv.len());
                let name = ast.get(f).get_value();
                if labels_of_arity.contains_key(&name) {
                    let label = labels_of_arity.get(&name).unwrap();
                    if label.is_inbuilt() {
                        if literals_only {
                            return Some(
                                labels_of_arity
                                    .get(&name)
                                    .unwrap()
                                    .call_inbuilt(ast.get(f), argv),
                            );
                        } else {
                            return None;
                        }
                    } else {
                        if !(ast.get(f).wait_for_args && literals_only) {
                            let assign = *am.get(&name).unwrap();
                            let assign_exp = ast.get_assign_exp(assign);
                            return Some(ast.do_multiple_abst_substs(assign_exp, argv_ids));
                        } else {
                            return None;
                        }
                    }
                } else {
                    return None;
                }
            }
            ASTNodeType::Abstraction => {
                if !(ast.get(f).wait_for_args && literals_only) {
                    return Some(ast.do_multiple_abst_substs(f, argv_ids));
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

pub fn find_redex_contraction_pairs(
    ast: &AST,
    module: usize,
    exp: usize,
    lt: &LabelTable,
) -> Vec<(usize, AST)> {
    let mut pairs: Vec<(usize, AST)> = vec![];

    #[cfg(debug_assertions)]
    let _exp_str = ast.to_string(exp);

    // Dont need to worry about this as main must be at the end, so everything defined in
    // the module is defined here
    let am: HashMap<String, usize> = ast.get_assigns_map(module);

    match ast.get(exp).t {
        ASTNodeType::Identifier => {
            let value = ast.get(exp).get_value();

            // It should not be non zero_ary func as otherwise it would be caught by the app case
            if lt.get_n_ary_labels(0).contains_key(&value) {
                let label = lt.get_n_ary_labels(0).get(&value).unwrap();

                if label.is_inbuilt() {
                    let inbuilt = label.call_inbuilt(&ast.get(exp), vec![]);
                    pairs.push((exp, inbuilt));
                } else {
                    let assign = *am.get(&value).unwrap();
                    let assign_exp = ast.get_assign_exp(assign);
                    pairs.push((exp, ast.clone_node(assign_exp)));
                }
            }
        }
        ASTNodeType::Application => {
            if let Some(inbuilt_reduction) = check_for_ready_call(ast, exp, &lt, am) {
                pairs.push((exp, inbuilt_reduction));
            }

            let f = ast.get_func(exp);
            let x = ast.get_arg(exp);

            #[cfg(debug_assertions)]
            let _f_str = ast.to_string(f);
            #[cfg(debug_assertions)]
            let _x_str = ast.to_string(x);
            match ast.get(f).t {
                ASTNodeType::Application | ASTNodeType::Identifier => {
                    pairs.extend(find_redex_contraction_pairs(ast, module, f, &lt));
                }
                // ASTNodeType::Abstraction => pairs.push((exp, ast.do_abst_subst(f, x))),
                ASTNodeType::Literal | ASTNodeType::Abstraction => {}
                _ => unreachable!("Expected expression"),
            }

            match ast.get(x).t {
                ASTNodeType::Application | ASTNodeType::Identifier => {
                    pairs.extend(find_redex_contraction_pairs(ast, module, f, &lt));
                }
                // ASTNodeType::Abstraction => pairs.push((exp, ast.do_abst_subst(f, x))),
                ASTNodeType::Literal | ASTNodeType::Abstraction => {}
                _ => unreachable!("Expected expression"),
            }

            pairs.extend(find_redex_contraction_pairs(
                ast,
                module,
                ast.get_arg(exp),
                &lt,
            ));
        }
        _ => {}
    }

    pairs
}
