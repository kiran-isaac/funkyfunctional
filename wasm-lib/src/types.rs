use std::{collections::HashMap, hash::Hash};

type TypeID = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Primitive {
  Invalid,
  
  Char,
  Short,
  Int,
  Long,

  Float,
  Double,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
  Primitive(Primitive),
  List(String),
  Alias(String),
  Tuple(Vec<TypeID>),

  // String label
  // EG : type Result T = (T, String)
  GenericParameter(String),

  // Input type, Output type
  Function(TypeID, TypeID)
}

struct TypeTable {
  map : HashMap<TypeID, Type>
}

impl TypeTable {
  fn new() -> TypeTable {
    let mut map: HashMap<TypeID, Type> = HashMap::new();

    map.insert(0, Type::Primitive(Primitive::Invalid));

    map.insert(1, Type::Primitive(Primitive::Char));
    map.insert(2, Type::Primitive(Primitive::Short));
    map.insert(3, Type::Primitive(Primitive::Int));
    map.insert(4, Type::Primitive(Primitive::Long));

    map.insert(5, Type::Primitive(Primitive::Float));
    map.insert(6, Type::Primitive(Primitive::Double));

    TypeTable {
      map : HashMap::new()
    }
  }
  
  pub fn get(&self, id : TypeID) -> Option<&Type> {
    self.map.get(&id)
  }

  pub fn insert(&mut self, ty : Type) {
    let id = self.map.len() as TypeID;
    self.map.insert(id, ty);
  }
}