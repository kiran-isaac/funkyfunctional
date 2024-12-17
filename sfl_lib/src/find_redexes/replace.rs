use std::collections::HashSet;

use crate::inbuilts::get_starting_bindings;

use super::*;

pub fn get_replacement_targets(
    mod_ast: &AST,
    exp_ast: &AST,
    exp: usize,
) -> Vec<(usize, usize)> {
    let inbuilts = get_starting_bindings();
    let assign_map = mod_ast.get_assign_map(mod_ast.root);

    let mut pairs: Vec<(usize, usize)> = vec![];

    match exp_ast.get(exp).t {
        ASTNodeType::Literal => {}
        ASTNodeType::Identifier => {
            for (name, target) in assign_map {
                if exp_ast.get(exp).get_value() == name {
                    pairs.push((exp, target));
                }
            }
        }
        ASTNodeType::Application => {
            let left = exp_ast.get_func(exp);
            let right = exp_ast.get_arg(exp);

            let left_pairs = get_replacement_targets(mod_ast, exp_ast, left);
            let right_pairs = get_replacement_targets(mod_ast, exp_ast, right);

            for (l, r) in left_pairs {
                pairs.push((l, r));
            }

            for (l, r) in right_pairs {
                pairs.push((l, r));
            }
        }
        _ => {
            unreachable!()
        }
    }

    pairs
}

pub fn do_replacement(
    mod_ast: &AST,
    exp_ast: &mut AST,
    replacement: (usize, usize),
) {
    exp_ast.replace(mod_ast, replacement.0, replacement.1);
}