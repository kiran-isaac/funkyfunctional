use std::{fmt::format, iter::Map};
use std::collections::HashSet;

use super::*;

pub struct ReplacementFinder;

impl ReplacementFinder {
    fn get_unique_ids(&self, root : &ASTNode) -> HashSet<String> {
        let mut ids = HashSet::new();
        match &root.t {
            ASTNodeType::Literal => {ids}
            ASTNodeType::Identifier => {
                ids.insert(root.get_value());
                ids
            }
            ASTNodeType::Application => {
                let mut left_ids = self.get_unique_ids(&root.get_func());
                let right_ids = self.get_unique_ids(&root.get_arg());
                
                left_ids.extend(right_ids);
                left_ids
            }
            _ => {unreachable!()}
        }
    }
    
    fn get_replacement_candidates(&self, root : &ASTNode, n : &str) -> Vec<ASTNode> {
        match &root.t {
            ASTNodeType::Literal => {vec![]}
            ASTNodeType::Identifier => {
                if n == root.get_value() {
                    vec![root.clone()]
                } else {vec![]}
            }
            ASTNodeType::Application => {
                let mut left_candidates = self.get_replacement_candidates(&root.get_func(), n);
                let mut right_candidates = self.get_replacement_candidates(&root.get_arg(), n);
                
                left_candidates.append(&mut right_candidates);
                left_candidates
            }
            _ => {unreachable!()}
        }
    }

    pub fn new() -> Self {
        Self {bindings : crate::inbuilts::get_starting_bindings()}
    }
    
    pub fn find(&self, root : &ASTNode) -> Result<Vec<ASTNode>, ()> {
        let mut candidates = vec![];

        for assign_name in root.get_assigns() {
            let ids = self.get_unique_ids(root);

            // check bindigs 
            for id in ids {
                let replacements = self.get_replacement_candidates(root, &id);
                candidates.extend(replacements);
            }
        }

        Ok(candidates)
    }
}