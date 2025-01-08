mod type_checker;
use std::collections::HashMap;

pub use type_checker::TypeChecker;

#[cfg(test)]
mod type_checker_test;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    Invalid,

    Int64,
    Float64,
    Bool,
    Unit
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Function(Box<Type>, Box<Type>),
    TypeVariable(usize),
}

pub struct TypeError {
    pub e: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Debug for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Type Error at [{}:{}]: {}",
            self.line + 1,
            self.col + 1,
            self.e
        )
    }
}

impl Type {
    pub fn int64() -> Type {
        Type::Primitive(Primitive::Int64)
    }

    pub fn float64() -> Type {
        Type::Primitive(Primitive::Float64)
    }

    pub fn bool() -> Type {
        Type::Primitive(Primitive::Bool)
    }

    // pub fn char() -> Type {
    //     Type::Primitive(Primitive::Char)
    // }

    pub fn f(t1: Type, t2: Type) -> Type {
        Type::Function(Box::new(t1), Box::new(t2))
    }

    pub fn g(usize: usize) -> Type {
        Type::TypeVariable(usize)
    }

    pub fn is_concrete(&self) -> bool {
        match self {
            Type::Primitive(_) => true,
            Type::Function(t1, t2) => t1.is_concrete() && t2.is_concrete(),
            Type::TypeVariable(_) => false,
        }
    }

    pub fn concrete_eq(&self, other: &Type) -> bool {
        #[cfg(debug_assertions)]
        {
            assert!(self.is_concrete());
            assert!(other.is_concrete());
        }
        match (self, other) {
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (Type::Function(t1, t2), Type::Function(t3, t4)) => {
                t1.concrete_eq(t3) && t2.concrete_eq(t4)
            }
            (Type::TypeVariable(_), _) | (_, Type::TypeVariable(_)) => unreachable!(),
            _ => false,
        }
    }

    fn max_type_var(&self) -> usize {
        match self {
            Type::Primitive(_) => 0,
            Type::Function(t1, t2) => std::cmp::max(t1.max_type_var(), t2.max_type_var()),
            Type::TypeVariable(n) => *n,
        }
    }

    fn add_to_type_vars(&self, increment: usize) -> Self {
        match self {
            Type::Primitive(p) => Type::Primitive(*p),
            Type::Function(t1, t2) => {
                Type::Function(
                    Box::new(t1.add_to_type_vars(increment)),
                    Box::new(t2.add_to_type_vars(increment)),
                )
            }
            Type::TypeVariable(n) => Type::TypeVariable(n + increment),
        }
    }

    /// Make sure that t1 and t2 dont have overlapping type variables
    pub fn ensure_different_type_params(t1: Type, t2: Type) -> (Type, Type) {
        let t2 = t2.add_to_type_vars(t1.max_type_var() + 1);
        (t1, t2)
    }

    pub fn get_arity(&self) -> usize {
        match self {
            Type::Primitive(_) => 0,
            Type::Function(_, t) => 1 + t.get_arity(),
            Type::TypeVariable(_) => 0,
        }
    }

    fn to_string_internal(&self, full_braces: bool) -> String {
        match self {
            Type::Primitive(p) => match p {
                Primitive::Int64 => "Int".to_string(),
                Primitive::Float64 => "Float".to_string(),
                Primitive::Bool => "Bool".to_string(),
                _ => unimplemented!(),
            },
            Type::Function(t1, t2) => {
                let t1_string = t1.to_string_internal(full_braces);
                let t1_string = match t1.as_ref() {
                    Type::Function(_, _) => format!("({})", t1_string),
                    _ => t1_string.clone(),
                };

                let mut t2_string = t2.to_string_internal(full_braces);
                if full_braces {
                    t2_string = match t2.as_ref() {
                        Type::Function(_, _) => format!("({})", t2_string),
                        _ => t2_string.clone(),
                    };
                }

                format!("{} -> {}", t1_string, t2_string)
            }
            Type::TypeVariable(n) => {
                let mut s = String::new();
                let mut n = *n;
                s.insert(0, (b'a' + (n % 26) as u8) as char);
                n /= 26;
                while n > 0 {
                    s.insert(0, (b'a' + (n % 26 - 1) as u8) as char);
                    n /= 26;
                }
                s
            }
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        self.to_string_internal(false)
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_string_internal(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_to_string() {
        let t1 = Type::Primitive(Primitive::Int64);
        assert_eq!(t1.to_string(), "Int");

        let t2 = Type::Primitive(Primitive::Float64);
        assert_eq!(t2.to_string(), "Float");

        let t3 = Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Primitive(Primitive::Float64)),
        );
        assert_eq!(t3.to_string(), "Int -> Float");

        let t4 = Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Float64)),
            )),
        );
        assert_eq!(t4.to_string(), "Int -> Int -> Float");

        let t5 = Type::Function(
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
            Box::new(Type::Primitive(Primitive::Float64)),
        );
        assert_eq!(t5.to_string(), "(Int -> Int) -> Float");

        let t6 = Type::TypeVariable(0);
        assert_eq!(t6.to_string(), "a");

        println!("{:?}", t6);

        let t6 = Type::TypeVariable(26);
        assert_eq!(t6.to_string(), "aa");

        let t6 = Type::TypeVariable(27);
        assert_eq!(t6.to_string(), "ab");

        let t6 = Type::TypeVariable(28);
        assert_eq!(t6.to_string(), "ac");

        let t6 = Type::TypeVariable(26 * 2);
        assert_eq!(t6.to_string(), "ba");
    }
}
