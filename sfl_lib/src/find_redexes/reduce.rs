use super::*;
use crate::find_redexes::pattern_match::pattern_match;
use crate::functions::KnownTypeLabelTable;
use std::collections::HashMap;

fn comma_ify(vec: Vec<String>) -> String {
    match vec.len() {
        0 => String::new(),
        1 => vec[0].clone(),
        _ => {
            let mut str = vec[0].clone();
            let last = vec[vec.len() - 1].clone();
            for string in &vec[1..vec.len() - 1] {
                str += ", ";
                str += string;
            }
            str + " and " + &last
        }
    }
}

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
fn check_for_valid_call(
    ast: &AST,
    expr: usize,
    lt: &KnownTypeLabelTable,
    am: HashMap<String, usize>,
) -> Option<RCPair> {
    let mut f = ast.get_func(expr);
    let mut x = ast.get_arg(expr);
    let mut argv = vec![];
    let mut argv_ids = vec![];

    let mut argv_strs = vec![];

    // True if only literals encountered. If true, then we can call inbuilt functions
    let mut literals_only = true;

    loop {
        argv.push(ast.get(x));
        argv_ids.push(x);

        argv_strs.push(ast.to_string_sugar(x, false));

        match ast.get(x).t {
            ASTNodeType::Literal => {}
            _ => literals_only = false,
        }
        let f_node = ast.get(f);

        match f_node.t {
            ASTNodeType::Identifier => {
                let labels_of_arity = if let Some(labels) = lt.get_n_ary_labels(argv.len()) {
                    labels
                } else {
                    return None;
                };
                let name = f_node.get_value();

                return if labels_of_arity.contains_key(&name) {
                    let label = labels_of_arity.get(&name).unwrap();
                    let argv_comma_str = comma_ify(argv_strs);
                    if label.is_inbuilt() {
                        if literals_only {
                            Some(
                                RCPair {
                                    from: expr,
                                    to: labels_of_arity
                                        .get(&name)
                                        .unwrap()
                                        .call_inbuilt(f_node, argv),
                                    msg_after: format!("Applied inbuilt {} to {}", name, &argv_comma_str),
                                    msg_before: format!("Apply inbuilt {} to {}", name, &argv_comma_str),
                                }

                            )
                        } else {
                            None
                        }
                    } else {
                        let assign = match am.get(&name) {
                            Some(a) => *a,
                            None => return None,
                        };

                        let assign_exp = ast.get_assign_exp(assign);
                        let n_args = ast.get_n_abstr_vars(assign_exp, argv.len());

                        if ast.get_n_abstr_vars(assign_exp, argv.len()).len() != argv.len() {
                            return None;
                        }

                        // Stop it being a ready call when a pair is expected but we dont have it
                        for i in 0..argv.len() {
                            match (&argv[i].t, &ast.get(n_args[i]).t) {
                                (ASTNodeType::Pair, ASTNodeType::Pair) => {}
                                (_, ASTNodeType::Pair) => return None,
                                _ => {}
                            }
                        }

                        let call_result =
                            ast.do_multiple_abst_substs(assign_exp, argv_ids);

                        #[cfg(debug_assertions)]
                        let _ready_call_result_str =
                            call_result.to_string_sugar(call_result.root, false);

                        Some(RCPair {
                            from: expr,
                            to: call_result,
                            msg_after: format!("Applied function {} to {}", name, &argv_comma_str),
                            msg_before: format!("Applied function {} to {}", name, &argv_comma_str)
                        })
                    }
                } else {
                    None
                };
            }
            ASTNodeType::Abstraction => {

            }
            ASTNodeType::Application => {
                x = ast.get_arg(f);
                f = ast.get_func(f);
            }
            _ => return None,
        }
    }
}

pub fn find_all_redex_contraction_pairs(
    ast: &AST,
    module: Option<usize>,
    expr: usize,
    lt: &KnownTypeLabelTable,
) -> Vec<RCPair> {
    let mut pairs: Vec<RCPair> = vec![];

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
        ASTNodeType::Application => {
            let f = ast.get_func(expr);
            let x = ast.get_arg(expr);

            if let Some(inbuilt_reduction) = check_for_valid_call(ast, expr, &lt, am) {
                pairs.push(inbuilt_reduction);
            }

            pairs.extend(find_all_redex_contraction_pairs(ast, module, f, &lt));
            pairs.extend(find_all_redex_contraction_pairs(ast, module, x, &lt));
        }
        ASTNodeType::Pair => {
            let left_rcs = find_all_redex_contraction_pairs(ast, module, ast.get_first(expr), lt);
            let right_rcs = find_all_redex_contraction_pairs(ast, module, ast.get_second(expr), lt);
            pairs.extend(left_rcs);
            pairs.extend(right_rcs);
        }
        ASTNodeType::Match | ASTNodeType::Identifier => {
            if let Some(rc) = find_single_redex_contraction_pair(ast, module, expr, lt) {
                pairs.push(rc);
            }
        }
        _ => panic!("Expected expression"),
    }

    pairs
}

pub fn find_single_redex_contraction_pair(
    ast: &AST,
    module: Option<usize>,
    expr: usize,
    lt: &KnownTypeLabelTable,
) -> Option<RCPair> {
    #[cfg(debug_assertions)]
    let _exp_str = ast.to_string_sugar(expr, false);

    // Dont need to worry about this as main must be at the end, so everything defined in
    // the module is defined here
    let am: HashMap<String, usize> = match module {
        Some(m) => ast.get_assigns_map(m),
        None => HashMap::new(),
    };

    match ast.get(expr).t {
        ASTNodeType::Literal | ASTNodeType::Abstraction => None,
        ASTNodeType::Pair => {
            if let Some(left_rc) =
                find_single_redex_contraction_pair(ast, module, ast.get_first(expr), lt)
            {
                Some(left_rc)
            } else {
                find_single_redex_contraction_pair(ast, module, ast.get_second(expr), lt)
            }
        }
        ASTNodeType::Identifier => {
            let value = ast.get(expr).get_value();

            // It should not be non zero_ary func as otherwise it would be caught by the app case
            for i in 0..lt.get_max_arity() {
                if let Some(labels) = lt.get_n_ary_labels(i) {
                    if labels.contains_key(&value) {
                        let label = labels.get(&value).unwrap();

                        return if label.is_inbuilt() && i == 0 {
                            let subst_result = label.call_inbuilt(&ast.get(expr), vec![]);
                            Some(RCPair {
                                from: expr,
                                to: subst_result,
                                msg_after: format!("Substituted label {}", &value),
                                msg_before: format!("Substituted label {}", &value)
                            })
                        } else {
                            let assign = if let Some(assign) = am.get(&value) {
                                *assign
                            } else {
                                return None;
                            };

                            let assign_exp = ast.get_assign_exp(assign);
                            let subst_result = ast.clone_node(assign_exp);

                            Some(RCPair {
                                from: expr,
                                to: subst_result,
                                msg_after: format!("Substituted label {}", &value),
                                msg_before: format!("Substitute label {}", &value)
                            })
                        };
                    }
                }
            }
            None
        }
        ASTNodeType::Application => {
            if let Some(ready_call_reduction) = check_for_valid_call(ast, expr, &lt, am) {
                Some(ready_call_reduction)
            } else if let Some(f_rc) =
                find_single_redex_contraction_pair(ast, module, ast.get_func(expr), lt)
            {
                Some(f_rc)
            } else {
                find_single_redex_contraction_pair(ast, module, ast.get_arg(expr), lt)
            }
        }
        ASTNodeType::Match => {
            let unpack_expr = ast.get_match_unpack_pattern(expr);
            for (pattern, pattern_expr) in ast.get_match_cases(expr) {
                if let Some(bindings) = pattern_match(ast, unpack_expr, pattern) {
                    let case_str = ast.to_string_sugar(pattern, false);
                    let mut pat_expr_cloned = ast.clone_node(pattern_expr);
                    for (var, replacement) in bindings {
                        let replacement_appended = pat_expr_cloned.append(ast, replacement);
                        let usages = pat_expr_cloned
                            .get_all_free_instances_of_var_in_exp(pat_expr_cloned.root, &var);
                        for usage in usages {
                            pat_expr_cloned.replace_references_to_node(usage, replacement_appended);
                        }
                    }
                    return Some(RCPair {
                        from: expr,
                        to: pat_expr_cloned.clone_node(pat_expr_cloned.root),
                        msg_after: format!("Matched to pattern {}", case_str),
                        msg_before: format!("Match to pattern {}", case_str)
                    });
                }
            }
            find_single_redex_contraction_pair(ast, module, unpack_expr, lt)
        }
        _ => None,
    }
}
