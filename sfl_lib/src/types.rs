#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
  Invalid,
  
  Int64,
  Float64,
  Bool,
  Char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
  Primitive(Primitive),
  Function(Box<Type>, Box<Type>),
}

pub struct TypeError {
  pub e: String,
  pub line: usize,
  pub col: usize,
}

impl std::fmt::Debug for TypeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "Type Error at [{}:{}]: {}", self.line, self.col, self.e)
  }
}