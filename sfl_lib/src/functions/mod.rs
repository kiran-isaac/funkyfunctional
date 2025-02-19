use std::collections::HashMap;

use inbuilt_arith::*;

use crate::*;
mod inbuilt_arith;

#[cfg(test)]
mod test;
mod inbuilt_io;

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

#[derive(Clone, Debug)]
pub struct Label {
    /// Arity needed to reduce the function.
    pub inbuilt_reduction_arity: usize,

    inbuilt: Option<InbuiltFuncPointer>,
    pub label_type: Type,
    pub is_silent: bool,
}

impl Label {
    pub fn call_inbuilt(&self, call: &ASTNode, args: Vec<&ASTNode>) -> AST {
        assert!(self.inbuilt_reduction_arity == args.len());
        assert!(self.inbuilt.is_some());
        (self.inbuilt.unwrap())(call, args)
    }

    pub fn is_inbuilt(&self) -> bool {
        self.inbuilt.is_some()
    }

    pub fn get_type(&self) -> &Type {
        &self.label_type
    }
}

#[derive(Debug, Clone)]
pub struct KnownTypeLabelTable {
    /// Sorted by arity. So inbuilts[0] will be all inbuilts with arity 0
    /// inbuilts[1] will be all inbuilts with arity 1, etc.
    func_map: Vec<HashMap<String, Label>>,
}

impl KnownTypeLabelTable {
    pub fn new() -> Self {
        let mut s = Self {
            func_map: vec![HashMap::new()],
        };
        s.populate_inbuilts();
        s
    }

    pub fn get_max_arity(&self) -> usize {
        self.func_map.len()
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        for inbuilt_map in &self.func_map {
            if inbuilt_map.contains_key(name) {
                return Some(&inbuilt_map.get(name).unwrap().label_type);
            }
        }

        None
    }

    /// Arity here is the number of arguments the inbuilt function needs to reduce
    /// It is not necessarily the same as the number of arguments the function takes
    /// as the function may be curried, for example 'if' takes one bool argument to
    /// reduce but it has a type of Bool -> A -> A -> A
    fn add_inbuilt(
        &mut self,
        name: String,
        arity: usize,
        func: InbuiltFuncPointer,
        func_type: Type,
    ) {
        if arity >= self.func_map.len() {
            self.func_map.resize(arity + 1, HashMap::new());
        }

        self.func_map[arity].insert(
            name,
            Label {
                inbuilt_reduction_arity: arity,
                inbuilt: Some(func),
                label_type: func_type,
                is_silent: true, // no effect for inbuilts
            },
        );
    }

    pub fn add(&mut self, name: String, type_: Type, is_silent: bool) {
        let arity = type_.get_arity();

        if arity >= self.func_map.len() {
            self.func_map.resize(arity + 1, HashMap::new());
        }

        self.func_map[arity].insert(
            name,
            Label {
                inbuilt_reduction_arity: arity,
                inbuilt: None,
                label_type: type_,
                is_silent,
            },
        );
    }

    pub fn remove(&mut self, name: &String) -> bool {
        for inbuilt_map in &mut self.func_map {
            if inbuilt_map.contains_key(name) {
                inbuilt_map.remove(name);
                return true;
            }
        }
        false
    }

    pub fn consume_from_module(&mut self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
            let ass_n = &ast.get(assign);
            let proclaimed_type = match &ass_n.type_assignment {
                None => {
                    return Err(TypeError {
                        e: format!("Label {} has no type assignment", name),
                        line: ast.get(assign).line,
                        col: ast.get(assign).col,
                    })
                }
                Some(t) => t.clone(),
            };

            self.add(name.clone(), proclaimed_type, ass_n.is_silent);
        }

        Ok(())
    }

    pub fn get_n_ary_labels(&self, arity: usize) -> Option<&HashMap<String, Label>> {
        if arity < self.func_map.len() {
            Some(&self.func_map[arity])
        } else {
            None
        }
    }

    pub fn get(&self, arity: usize, name: String) -> Option<&Label> {
        self.get_n_ary_labels(arity)?.get(&name)
    }

    fn populate_inbuilts(&mut self) {
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

        self.add_inbuilt(
            "add".to_string(),
            2,
            inbuilt_int_add,
            binary_int_type.clone(),
        );
        self.add_inbuilt("+".to_string(), 2, inbuilt_int_add, binary_int_type.clone());
        self.add_inbuilt(
            "sub".to_string(),
            2,
            inbuilt_int_sub,
            binary_int_type.clone(),
        );
        self.add_inbuilt("-".to_string(), 2, inbuilt_int_sub, binary_int_type.clone());
        self.add_inbuilt(
            "mul".to_string(),
            2,
            inbuilt_int_mul,
            binary_int_type.clone(),
        );
        self.add_inbuilt("*".to_string(), 2, inbuilt_int_mul, binary_int_type.clone());
        self.add_inbuilt(
            "div".to_string(),
            2,
            inbuilt_int_div,
            binary_int_type.clone(),
        );
        self.add_inbuilt("/".to_string(), 2, inbuilt_int_div, binary_int_type.clone());
        self.add_inbuilt(
            "mod".to_string(),
            2,
            inbuilt_int_mod,
            binary_int_type.clone(),
        );
        self.add_inbuilt("%".to_string(), 2, inbuilt_int_mod, binary_int_type.clone());

        self.add_inbuilt(
            "eq".to_string(),
            2,
            inbuilt_int_eq,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "==".to_string(),
            2,
            inbuilt_int_eq,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "lte".to_string(),
            2,
            inbuilt_int_lte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "<=".to_string(),
            2,
            inbuilt_int_lte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "lt".to_string(),
            2,
            inbuilt_int_lt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "<".to_string(),
            2,
            inbuilt_int_lt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "gte".to_string(),
            2,
            inbuilt_int_gte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            ">=".to_string(),
            2,
            inbuilt_int_gte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "gt".to_string(),
            2,
            inbuilt_int_gt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(">".to_string(), 2, inbuilt_int_gt, binary_int_bool_type);

        self.add_inbuilt(
            "addf".to_string(),
            2,
            inbuilt_float_add,
            binary_float_type.clone(),
        );
        self.add_inbuilt(
            "subf".to_string(),
            2,
            inbuilt_float_sub,
            binary_float_type.clone(),
        );
        self.add_inbuilt(
            "mulf".to_string(),
            2,
            inbuilt_float_mul,
            binary_float_type.clone(),
        );
        self.add_inbuilt("divf".to_string(), 2, inbuilt_float_div, binary_float_type);
        self.add_inbuilt(
            "eqf".to_string(),
            2,
            inbuilt_float_eq,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "ltef".to_string(),
            2,
            inbuilt_float_lte,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "ltf".to_string(),
            2,
            inbuilt_float_lt,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "gtef".to_string(),
            2,
            inbuilt_float_gte,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "gtf".to_string(),
            2,
            inbuilt_float_gt,
            binary_float_bool_type,
        );

        // self.add_inbuilt("if".to_string(), 1, inbuilt_if, if_type);

        self.add_inbuilt("neg".to_string(), 1, inbuilt_int_neg, unary_int_type);
        self.add_inbuilt("negf".to_string(), 1, inbuilt_float_neg, unary_float_type);

        #[cfg(test)]
        self.add_inbuilt(
            "zero_ary_test".to_string(),
            0,
            inbuilt_int_zero,
            Type::int64(),
        );

        self.add_inbuilt(
            "putInt",
            1,

            IO_TYPE.type_app(&Type::int64()),
        )
    }

    /// Get all strings that are inbuilts so that they can be added to the bound checker
    pub fn get_starting_bindings_map() -> Vec<String> {
        let mut bindings = vec![];
        for inbuilt_map in &KnownTypeLabelTable::new().func_map {
            for inbuilt in inbuilt_map.keys() {
                bindings.push(inbuilt.clone());
            }
        }

        bindings
    }

    pub fn get_type_map(&self) -> HashMap<String, Type> {
        let mut type_map = HashMap::new();
        for inbuilt_map in &self.func_map {
            for (name, inbuilt) in inbuilt_map {
                type_map.insert(name.clone(), inbuilt.label_type.clone());
            }
        }

        type_map
    }

    pub fn get_non_builtin_type_map(&self) -> HashMap<String, Type> {
        let builtin_type_map = Self::new().get_type_map();
        let mut type_map = HashMap::new();
        for inbuilt_map in &self.func_map {
            for (name, inbuilt) in inbuilt_map {
                if builtin_type_map.contains_key(name) {
                    continue;
                }
                type_map.insert(name.clone(), inbuilt.label_type.clone());
            }
        }

        type_map
    }
}
