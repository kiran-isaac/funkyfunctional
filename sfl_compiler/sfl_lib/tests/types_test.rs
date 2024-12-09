use sfl_lib::types::{Primitive, TypeConstructorArg, Type, TypeTable};

#[test]
fn List() {
  let list_nil = ("Nil", vec![]);
  let list_cons = ("Cons", vec![TypeConstructorArg::Generic(0), TypeConstructorArg::Recursive(vec![TypeConstructorArg::Generic(0)])]););

  let mut tt = TypeTable::new();
  let list_type = Type::new_non_primitive("List", vec!["T"], vec![list_nil, list_cons]);
  tt.add_type(list_type);
}

#[test]
fn MainFunction() {
  let mut tt = TypeTable::new();
}