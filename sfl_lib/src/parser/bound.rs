use crate::functions::KnownTypeLabelTable;
use std::collections::HashSet;

pub struct BoundChecker {
    bound: HashSet<String>,
}

impl BoundChecker {
    pub fn new() -> Self {
        let mut bound = HashSet::new();

        for binding in KnownTypeLabelTable::get_starting_bindings_map() {
            bound.insert(binding);
        }

        Self { bound }
    }

    pub fn add_binding(&mut self, name: String) {
        self.bound.insert(name);
    }

    pub fn remove_binding(&mut self, name: String) {
        self.bound.remove(&name);
    }

    pub fn is_bound(&self, name: &str) -> bool {
        self.bound.contains(name)
    }

    pub fn append(&mut self, other: &BoundChecker) {
        for binding in &other.bound {
            self.bound.insert(binding.clone());
        }
    }
}
