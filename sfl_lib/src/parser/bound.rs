use std::collections::HashSet;
use crate::inbuilts::get_starting_bindings;

pub struct BoundChecker {
    bound: HashSet<String>,
}

impl BoundChecker {
    pub fn new() -> Self {
        let mut bound = HashSet::new();
        
        for binding in get_starting_bindings() {
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
}
