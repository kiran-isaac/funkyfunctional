use std::collections::HashMap;

use arith::*;
use control_flow::*;

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
        self.add_inbuilt("eq".to_string(), 2, inbuilt_int_eq);
        self.add_inbuilt("lte".to_string(), 2, inbuilt_int_lte);
        self.add_inbuilt("lt".to_string(), 2, inbuilt_int_lt);
        self.add_inbuilt("gte".to_string(), 2, inbuilt_int_gte);
        self.add_inbuilt("gt".to_string(), 2, inbuilt_int_gt);

        self.add_inbuilt("addf".to_string(), 2, inbuilt_float_add);
        self.add_inbuilt("subf".to_string(), 2, inbuilt_float_sub);
        self.add_inbuilt("mulf".to_string(), 2, inbuilt_float_mul);
        self.add_inbuilt("divf".to_string(), 2, inbuilt_float_div);
        self.add_inbuilt("eqf".to_string(), 2, inbuilt_float_eq);
        self.add_inbuilt("ltef".to_string(), 2, inbuilt_float_lte);
        self.add_inbuilt("ltf".to_string(), 2, inbuilt_float_lt);
        self.add_inbuilt("gtef".to_string(), 2, inbuilt_float_gte);
        self.add_inbuilt("gtf".to_string(), 2, inbuilt_float_gt);

        self.add_inbuilt("const".to_string(), 2, inbuilt_const);
        self.add_inbuilt("unconst".to_string(), 2, inbuilt_unconst);

        self.add_inbuilt("if".to_string(), 1, inbuilt_if);
        self.add_inbuilt("iff".to_string(), 1, inbuilt_if);

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
    let binary_int_bool_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Int64)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Primitive(Primitive::Bool)),
        )),
    );
    let binary_float_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Float64)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Primitive(Primitive::Float64)),
        )),
    );
    let binary_float_bool_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Float64)),
        Box::new(Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Primitive(Primitive::Bool)),
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

    let if_type = Type::Function(
        Box::new(Type::Primitive(Primitive::Bool)),
        Box::new(Type::Function(
            Box::new(Type::g(0)),
            Box::new(Type::Function(
                Box::new(Type::g(0)),
                Box::new(Type::g(0)),
            )),
        )),
    );

    inbuilt_type_map.insert(
        "const".to_string(),
        Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
        ),
    );

    inbuilt_type_map.insert(
        "unconst".to_string(),
        Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
        ),
    );

    inbuilt_type_map.insert("if".to_string(), if_type);

    inbuilt_type_map.insert("add".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("sub".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("mul".to_string(), binary_int_type.clone());
    inbuilt_type_map.insert("div".to_string(), binary_int_type);
    inbuilt_type_map.insert("eq".to_string(), binary_int_bool_type.clone());
    inbuilt_type_map.insert("lte".to_string(), binary_int_bool_type.clone());
    inbuilt_type_map.insert("lt".to_string(), binary_int_bool_type.clone());
    inbuilt_type_map.insert("gte".to_string(), binary_int_bool_type.clone());
    inbuilt_type_map.insert("gt".to_string(), binary_int_bool_type);

    inbuilt_type_map.insert("addf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("subf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("mulf".to_string(), binary_float_type.clone());
    inbuilt_type_map.insert("divf".to_string(), binary_float_type);
    inbuilt_type_map.insert("eqf".to_string(), binary_float_bool_type.clone());
    inbuilt_type_map.insert("ltef".to_string(), binary_float_bool_type.clone());
    inbuilt_type_map.insert("ltf".to_string(), binary_float_bool_type.clone());
    inbuilt_type_map.insert("gtef".to_string(), binary_float_bool_type.clone());
    inbuilt_type_map.insert("gtf".to_string(), binary_float_bool_type);

    inbuilt_type_map.insert("neg".to_string(), unary_int_type.clone());
    inbuilt_type_map.insert("negf".to_string(), unary_float_type.clone());

    inbuilt_type_map
}
