use std::collections::HashMap;

use arith::*;
use control_flow::inbuilt_int_if;

use crate::*;
mod arith;
mod control_flow;

#[cfg(test)]
mod test;

fn assert_prim_type(x: &Type, p: Primitive) {
    match x {
        Type::Primitive(prim) => {
            if *prim != p {
                panic!("ASSERT_PRIM_TYPE failed: Invalid type, wrong primitive")
            }
        }
        _ => panic!("ASSERT_PRIM_TYPE failed: Invalid type, not a primitive"),
    }
}

type InbuiltFuncPointer = fn(&ASTNode, Vec<&ASTNode>) -> AST;

/// Will be used to store inbuilt functions and their arities. will eventually
/// have some sort of function pointer or something to the actual function
#[derive(Clone)]
pub struct InbuiltFunc {
    arity: usize,
    func: InbuiltFuncPointer,
}

impl InbuiltFunc {
    pub fn call(&self, call: &ASTNode, args: Vec<&ASTNode>) -> AST {
        assert!(self.arity == args.len());
        (self.func)(call, args)
    }
}

pub struct InbuiltsLookupTable {
    /// Sorted by arity. So inbuilts[0] will be all inbuilts with arity 0
    /// inbuilts[1] will be all inbuilts with arity 1, etc.
    inbuilts: Vec<HashMap<String, InbuiltFunc>>,
}

impl InbuiltsLookupTable {
    pub fn new() -> Self {
        let mut s = Self {
            inbuilts: vec![HashMap::new()],
        };
        s.populate();
        s
    }

    pub fn get_max_arity(&self) -> usize {
        self.inbuilts.len()
    }

    fn add_inbuilt(&mut self, name: String, arity: usize, func: InbuiltFuncPointer) {
        if arity >= self.inbuilts.len() {
            self.inbuilts.resize(arity + 1, HashMap::new());
        }

        self.inbuilts[arity].insert(name, InbuiltFunc { arity, func });
    }

    pub fn get_n_ary_inbuilts(&self, arity: usize) -> &HashMap<String, InbuiltFunc> {
        &self.inbuilts[arity]
    }

    #[cfg(test)]
    pub fn get(&self, arity: usize, name: String) -> Option<&InbuiltFunc> {
        self.get_n_ary_inbuilts(arity).get(&name)
    }

    fn populate(&mut self) {
        self.add_inbuilt("add".to_string(), 2, inbuilt_int_add);
        self.add_inbuilt("sub".to_string(), 2, inbuilt_int_sub);
        self.add_inbuilt("mul".to_string(), 2, inbuilt_int_mul);
        self.add_inbuilt("div".to_string(), 2, inbuilt_int_div);

        self.add_inbuilt("addf".to_string(), 2, inbuilt_float_add);
        self.add_inbuilt("subf".to_string(), 2, inbuilt_float_sub);
        self.add_inbuilt("mulf".to_string(), 2, inbuilt_float_mul);
        self.add_inbuilt("divf".to_string(), 2, inbuilt_float_div);

        self.add_inbuilt("if".to_string(), 1, inbuilt_int_if);

        self.add_inbuilt("neg".to_string(), 1, inbuilt_int_neg);
        self.add_inbuilt("negf".to_string(), 1, inbuilt_float_neg);

        #[cfg(test)]
        self.add_inbuilt("zero_ary_test".to_string(), 0, inbuilt_int_zero);
    }

    /// Get all strings that are inbuilts so that they can be added to the bound checker
    pub fn get_starting_bindings_map() -> Vec<String> {
        let mut bindings = vec![];
        for inbuilt_map in &InbuiltsLookupTable::new().inbuilts {
            for inbuilt in inbuilt_map.keys() {
                bindings.push(inbuilt.clone());
            }
        }

        bindings
    }
}

/// Will be included in the InbuiltsLookupTable eventually
pub fn get_default_inbuilt_type_map() -> HashMap<String, Type> {
    let mut inbuilt_type_map = HashMap::new();
    let binary_int_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Int64)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Primitive(Primitive::Int64)),
        )),
    );
    let binary_float_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Float64)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Primitive(Primitive::Float64)),
        )),
    );
    let unary_int_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Int64)),
        Box::new(Type::Primitive(Primitive::Int64)),
    );
    let unary_float_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Float64)),
        Box::new(Type::Primitive(Primitive::Float64)),
    );

    let if_int_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Bool)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
        )),
    );

    let if_float_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Bool)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
        )),
    );

    inbuilt_type_map.insert("if".to_string(), if_int_type);
    inbuilt_type_map.insert("iff".to_string(), if_float_type);

    inbuilt_type_map.insert("add".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("sub".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("mul".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("div".to_string(), binary_int_type);

    inbuilt_type_map.insert("addf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("subf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("mulf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("divf".to_string(), binary_float_type);

    inbuilt_type_map.insert("neg".to_string(), unary_int_type.clone());
    inbuilt_type_map.insert("negf".to_string(), unary_float_type.clone());

    inbuilt_type_map
}
