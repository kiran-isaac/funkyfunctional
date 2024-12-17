use std::{fmt::format, iter::Map};
use std::collections::HashSet;

use super::*;

fn get_unique_ids(root : &ASTNode) -> HashSet<String> {
    let mut ids = HashSet::new();
    match &root.t {
        ASTNodeType::Literal => {ids}
        ASTNodeType::Identifier => {
            ids.insert(root.get_value());
            ids
        }
        ASTNodeType::Application => {
            let mut left_ids = get_unique_ids(&root.get_func());
            let right_ids = get_unique_ids(&root.get_arg());
            
            left_ids.extend(right_ids);
            left_ids
        }
        _ => {unreachable!()}
    }
}

fn get_replacement_candidates(root : &ASTNode, n : &str) -> Vec<ASTNode> {
    match &root.t {
        ASTNodeType::Literal => {vec![]}
        ASTNodeType::Identifier => {
            if n == root.get_value() {
                vec![root.clone()]
            } else {vec![]}
        }
        ASTNodeType::Application => {
            let mut left_candidates = get_replacement_candidates(&root.get_func(), n);
            let mut right_candidates = get_replacement_candidates(&root.get_arg(), n);
            
            left_candidates.append(&mut right_candidates);
            left_candidates
        }
        _ => {unreachable!()}
    }
}

pub fn find(root : &ASTNode) -> Result<Vec<ASTNode>, ()> {
    let mut candidates = vec![];

    for assign_name in root.get_assigns() {
        let exp = root.get_assign_to(assign_name).unwrap().get_exp();
        let ids = get_unique_ids(exp);

        // check bindigs 
        for id in ids {
            let replacements = get_replacement_candidates(exp, &id);
            candidates.extend(replacements);
        }
    }

    Ok(candidates)
}
