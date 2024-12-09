use std::{collections::HashMap, hash::Hash};

type TypeID = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
  Invalid,
  
  Char,
  Short,
  Int,
  Long,

  Float,
  Double,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeKind {
  Primitive,
  List,
  Alias,
  Tuple,
  GenericParameter,
  Function,
}


#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeValue {
  Primitive(Primitive),
  List(Box<Type>),
  Alias(TypeID),
  Tuple(Vec<TypeID>),
  GenericParameter(TypeID),
  Function(Box<Type>, Box<Type>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Type {
  kindedness : usize,
  kind : TypeKind,
  name : String,
  value : TypeValue
}

impl Clone for Type {
  fn clone(&self) -> Type {
    Type {
      kind : self.kind.clone(),
      kindedness : self.kindedness,
      name : self.name.clone(),
      value : match &self.value {
        TypeValue::Primitive(p) => TypeValue::Primitive(*p),
        TypeValue::List(ty) => TypeValue::List(Box::new(*ty.clone())),
        TypeValue::Alias(id) => TypeValue::Alias(*id),
        TypeValue::Tuple(ids) => TypeValue::Tuple(ids.clone()),
        TypeValue::GenericParameter(id) => TypeValue::GenericParameter(*id),
        TypeValue::Function(ty1, ty2) => TypeValue::Function(Box::new(*ty1.clone()), Box::new(*ty2.clone()))
      }
    }
  }
}

impl Type {
  pub fn new_primitive(name : String, primitive : Primitive) -> Type {
    Type {
      kind : 0,
      kindedness : TypeKind::Primitive,
      name,
      value : TypeValue::Primitive(primitive)
    }
  }

  pub fn new_list(name : String, ty : Type) -> Type {
    Type {
      kind : 0,
      kindedness : TypeKind::List,
      name,
      value : TypeValue::List(Box::new(ty))
    }
  }
}

// struct TypeTable {
//   map : HashMap<TypeID, Type>
// }

// impl TypeTable {
//   fn new() -> TypeTable {
//     let mut map: HashMap<TypeID, Type> = HashMap::new();

//     map.insert(0, Type::Primitive(Primitive::Invalid));

//     map.insert(1, Type::Primitive(Primitive::Char));
//     map.insert(2, Type::Primitive(Primitive::Short));
//     map.insert(3, Type::Primitive(Primitive::Int));
//     map.insert(4, Type::Primitive(Primitive::Long));

//     map.insert(5, Type::Primitive(Primitive::Float));
//     map.insert(6, Type::Primitive(Primitive::Double));

//     TypeTable {
//       map : HashMap::new()
//     }
//   }
  
//   pub fn get(&self, id : TypeID) -> Option<&Type> {
//     self.map.get(&id)
//   }

//   pub fn insert(&mut self, ty : Type) {
//     let id = self.map.len() as TypeID;
//     self.map.insert(id, ty);
//   }
// }