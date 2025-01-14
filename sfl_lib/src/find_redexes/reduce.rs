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

    loop {
        argv.push(ast.get(x));
        argv_ids.push(x);
        match ast.get(x).t {
            ASTNodeType::Literal => {}
            _ => literals_only = false,
        }

        match ast.get(f).t {
            ASTNodeType::Identifier => {
                let labels_of_arity = if let Some(lables) = lt.get_n_ary_labels(argv.len()) {
                    lables
                } else {
                    return None;
                };
                let name = ast.get(f).get_value();

                if name == "second" {
                    let _y = 1 + 1;
                }

                return if labels_of_arity.contains_key(&name) {
                    let label = labels_of_arity.get(&name).unwrap();
                    if label.is_inbuilt() {
                        if literals_only {
                            Some(
                                labels_of_arity
                                    .get(&name)
                                    .unwrap()
                                    .call_inbuilt(ast.get(f), argv),
                            )
                        } else {
                            None
                        }
                    } else {
                        if !(ast.get(f).wait_for_args && literals_only) {
                            let assign = *am.get(&name).unwrap();

                            let assign_exp = ast.get_assign_exp(assign);
                            let n_args = ast.get_n_abstr_vars(assign_exp, argv.len());
                            assert_eq!(argv.len(), n_args.len());

                            for i in 0..argv.len() {
                                match (&argv[i].t, &ast.get(n_args[i]).t) {
                                    (ASTNodeType::Pair, ASTNodeType::Pair) => {},
                                    (_, ASTNodeType::Pair) => return None,
                                    _ => {}
                                }
                            }

                            Some(ast.do_multiple_abst_substs(assign_exp, argv_ids))
                        } else {
                            None
                        }
                    }
                } else {
                    None
                };
            }
            ASTNodeType::Abstraction => {
                return if !(ast.get(f).wait_for_args && literals_only) {
                    let n_args = ast.get_n_abstr_vars(f, argv.len());
                    assert_eq!(argv.len(), n_args.len());

                    for i in 0..argv.len() {
                        match (&argv[i].t, &ast.get(n_args[i]).t) {
                            (ASTNodeType::Pair, ASTNodeType::Pair) => {},
                            (_, ASTNodeType::Pair) => return None,
                            _ => {}
                        }
                    }

                    Some(ast.do_multiple_abst_substs(f, argv_ids))
                } else {
                    None
                }
            }
            ASTNodeType::Application => {
                x = ast.get_arg(f);
                f = ast.get_func(f);
            }
            _ => return None,
        }
    }
}

pub fn find_redex_contraction_pairs(
    ast: &AST,
    module: Option<usize>,
    expr: usize,
    lt: &LabelTable,
) -> Vec<(usize, AST)> {
    let mut pairs: Vec<(usize, AST)> = vec![];

    #[cfg(debug_assertions)]
    let _exp_str = ast.to_string_sugar(expr, false);

    // Dont need to worry about this as main must be at the end, so everything defined in
    // the module is defined here
    let am: HashMap<String, usize> = match module {
        Some(m) => ast.get_assigns_map(m),
        None => HashMap::new(),
    };

    match ast.get(expr).t {
        ASTNodeType::Literal | ASTNodeType::Abstraction => {}
        ASTNodeType::Pair => {
            let left_rcs = find_redex_contraction_pairs(ast, module, ast.get_first(expr), lt);
            let right_rcs = find_redex_contraction_pairs(ast, module, ast.get_second(expr), lt);
            pairs.extend(left_rcs);
            pairs.extend(right_rcs);
        }
        ASTNodeType::Identifier => {
            let value = ast.get(expr).get_value();

            // It should not be non zero_ary func as otherwise it would be caught by the app case
            if let Some(labels) = lt.get_n_ary_labels(0) {
                if labels.contains_key(&value) {
                    let label = labels.get(&value).unwrap();

                    if label.is_inbuilt() {
                        let inbuilt = label.call_inbuilt(&ast.get(expr), vec![]);
                        pairs.push((expr, inbuilt));
                    } else {
                        let assign = *am.get(&value).unwrap();
                        let assign_exp = ast.get_assign_exp(assign);
                        pairs.push((expr, ast.clone_node(assign_exp)));
                    }
                }
            } else {
                unreachable!("No label match: {}", value);
            }
        }
        ASTNodeType::Application => {
            let f = ast.get_func(expr);
            let x = ast.get_arg(expr);

            if let Some(inbuilt_reduction) = check_for_ready_call(ast, expr, &lt, am) {
                pairs.push((expr, inbuilt_reduction));
            }

            pairs.extend(find_redex_contraction_pairs(ast, module, f, &lt));
            pairs.extend(find_redex_contraction_pairs(ast, module, x, &lt));
        }
        _ => unreachable!("Expected expression"),
    }

    pairs
}
