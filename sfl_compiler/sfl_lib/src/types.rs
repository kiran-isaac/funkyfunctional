type TypeID = u64;

use std::{collections::HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
  Invalid,
  
  Int8,
  Int16,
  Int32,
  Int64,

  F16,
  F32,
  F64,
}

#[derive(Debug, Clone)]
pub enum TypeConstructorArg {
  // Another type
  Existing(TypeID, Vec<TypeConstructorArg>),
  // This type
  Recursive(Vec<TypeConstructorArg>),
  // Index of the generic parameter
  Generic(u64)
}

type TypeConstructor = Vec<TypeConstructorArg>;
type FunctionSignature = Vec<TypeID>;

#[derive(Debug, Clone)]
enum TypeValue {
  Primitive(Primitive),
  Alias(TypeID),
  NotPrimitive(HashMap<String, TypeConstructor>),
  Function(FunctionSignature),
}

#[derive(Debug, Clone)]
pub struct Type {
  name: String,
  generic_params: Vec<String>,

  value: TypeValue,
}

impl Type {
  pub fn new_non_primitive(name : &str, generic_params : Vec<&str>, constructors : Vec<(&str, Vec<TypeConstructorArg>)>) -> Self {
    let constructors = constructors.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

    Self {
      name : name.to_string(),
      generic_params : generic_params.into_iter().map(|n| n.to_string()).collect(),
      value : TypeValue::NotPrimitive(constructors)
    }
  }

  pub fn new_function(name : &str, signature : FunctionSignature) -> Self {
    Self {
      name : name.to_string(),
      generic_params : vec![],
      value : TypeValue::Function(signature),
    }
  }

  pub fn from_primitive(primitive : Primitive) -> Self {
    Self {
      name : "".to_string(),
      generic_params : vec![],
      value : TypeValue::Primitive(primitive)
    }
  }

  pub fn make_alias(name : &str, type_id : TypeID) -> Self {
    Self {
      name : name.to_string(),
      generic_params : vec![],
      value : TypeValue::Alias(type_id)
    }
  }

  pub fn id_from_primitive(primitive : Primitive) -> TypeID {
    match primitive {
      Primitive::Invalid => 0,
      Primitive::Int8 => 1,
      Primitive::Int16 => 2,
      Primitive::Int32 => 3,
      Primitive::Int64 => 4,
      Primitive::F16 => 5,
      Primitive::F32 => 6,
      Primitive::F64 => 7,
    }
  }

  pub fn fill_generic(&self, type_constructor_arg : TypeConstructorArg) -> Result<Type, ()> {
    match &self.value {
      TypeValue::Alias(_) | TypeValue::Primitive(_) | TypeValue::Function(_) => {
        return Err(());
      },
      TypeValue::NotPrimitive(_) => {}
    }

    let cloned = self.clone();

    for (i, param) in self.generic_params.iter().enumerate() {
      match type_constructor_arg {
        TypeConstructorArg::Generic(n) if n == i as u64 => {
          return Ok(cloned);
        },
        _ => {}
      }
    }

    Ok(cloned)
  }
}

// eg List a = Cons a (List a) | Nil
// would be represented as
/*
Type {
  name: "List",
  generic_params: ["a"],
  constructors: {
    "Cons": [[Generic(0), Recursive(Generic(0))]],
    "Nil": []
  }
}
*/

pub struct TypeTable {
  types : Vec<Type>
}

impl TypeTable {
  pub fn new() -> Self {
    let mut man = Self {
      types : Vec::new()
    };

    man.add_type(Type::from_primitive(Primitive::Invalid));

    man.add_type(Type::from_primitive(Primitive::Int8));
    man.add_type(Type::from_primitive(Primitive::Int16));
    man.add_type(Type::from_primitive(Primitive::Int32));
    man.add_type(Type::from_primitive(Primitive::Int64));

    man.add_type(Type::from_primitive(Primitive::F16));
    man.add_type(Type::from_primitive(Primitive::F32));
    man.add_type(Type::from_primitive(Primitive::F64));

    let unit = Type::new_non_primitive("Unit", vec![], vec![("()", vec![])]);
    let list = Type::new_non_primitive("List", vec!["T"], vec![("Nil", vec![]), ("Cons", vec![TypeConstructorArg::Generic(0), TypeConstructorArg::Recursive(vec![TypeConstructorArg::Generic(0)])])]);

    man
  }

  pub fn len(&self) -> usize {
    self.types.len()
  }

  pub fn add_type(&mut self, type_ : Type) -> TypeID {
    let id = self.types.len() as TypeID;
    self.types.push(type_);
    id
  }
  
  pub fn get_type(&self, id : TypeID) -> Result<Type, ()> {
    if (self.types.len() as TypeID) <= id {
      return Err(());
    }

    Ok(self.types[id as usize].clone())
  }
}