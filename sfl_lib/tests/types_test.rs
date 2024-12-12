// use sfl_lib::types::{Primitive, TypeConstructorArg, Type, TypeTable};

// #[test]
// fn list() {
//   let list_nil = ("Nil", vec![]);
//   let list_cons = ("Cons", vec![TypeConstructorArg::Generic(0), TypeConstructorArg::Recursive(vec![TypeConstructorArg::Generic(0)])]);

//   let mut tt = TypeTable::new();
//   let list_type = Type::new_non_primitive("List", vec!["T"], vec![list_nil, list_cons]);
//   let id = tt.add_type(list_type);
//   println!("{}", tt.get_type(id).unwrap().to_string());
// }

// #[test]
// fn main_function() {
//   let mut tt = TypeTable::new();
// }
