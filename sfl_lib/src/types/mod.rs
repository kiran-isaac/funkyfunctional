mod type_checker;

use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
pub use type_checker::*;

#[cfg(test)]
mod type_checker_test;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    Invalid,

    Int64,
    Float64,
    Bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Primitive(Primitive),
    Function(Box<Type>, Box<Type>),
    TypeVariable(usize),
    Existential(usize),
    Forall(usize, Box<Type>),
}

#[derive(Clone, PartialEq, Eq)]
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

    pub fn f(t1: Type, t2: Type) -> Type {
        Type::Function(Box::new(t1), Box::new(t2))
    }

    pub fn g(usize: usize) -> Type {
        Type::TypeVariable(usize)
    }

    pub fn fa(forall: Vec<usize>, t: Self) -> Self {
        let mut t = t;
        for i in forall.into_iter().rev() {
            t = Type::Forall(i, Box::new(t));
        }
        t
    }

    pub fn contains_existential(&self, ex: usize) -> bool {
        match self {
            Type::Primitive(_) => false,
            Type::Function(t1, t2) => t1.contains_existential(ex) || t2.contains_existential(ex),
            Type::TypeVariable(_) => false,
            Type::Existential(e) => *e == ex,
            Type::Forall(_, t) => t.contains_existential(ex),
            Type::Unit => false,
        }
    }

    pub fn substitute_type_variable(&self, var: usize, replacement: &Type) -> Result<Self, String> {
        match self {
            Type::Primitive(p) => Ok(Type::Primitive(*p)),
            Type::Function(t1, t2) => Ok(Type::Function(
                Box::new(t1.substitute_type_variable(var, replacement)?),
                Box::new(t2.substitute_type_variable(var, replacement)?),
            )),
            Type::TypeVariable(n) => {
                if *n == var {
                    Ok(replacement.clone())
                } else {
                    Ok(Type::TypeVariable(*n))
                }
            }
            Type::Forall(var2, t2) => {
                if *var2 == var {
                    panic!("Duplicate forall")
                }
                Ok(Type::fa(
                    vec![*var2],
                    t2.substitute_type_variable(var, replacement)?,
                ))
            }
            _ => Ok(self.clone()),
        }
    }

    fn remove_duplicates<T : Eq + Hash + Clone>(ls : &Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        let mut new_vec : Vec<T> = Vec::new();
        for i in ls {
            if !seen.contains(i) {
                new_vec.push(i.clone());
                seen.insert(i);
            }
        }
        new_vec
    }

    fn ordered_tvs(&self) -> Vec<usize> {
        match &self {
            Type::TypeVariable(n) => vec![*n],
            Type::Forall(_, t2) => t2.ordered_tvs(),
            Type::Function(t1, t2) => {
                let mut t1 = t1.ordered_tvs();
                let t2 = t2.ordered_tvs();
                t1.extend(t2);
                Self::remove_duplicates(&t1)
            }
            _ => vec![],
        }
    }

    fn ordered_existentials(&self) -> Vec<usize> {
        match &self {
            Type::Existential(n) => vec![*n],
            Type::Forall(_, t2) => t2.ordered_existentials(),
            Type::Function(t1, t2) => {
                let mut t1 = t1.ordered_existentials();
                let t2 = t2.ordered_existentials();
                t1.extend(t2);
                Self::remove_duplicates(&t1)
            }
            _ => vec![],
        }
    }

    fn exist_to_tv(&self, max: usize) -> Self {
        match self {
            Type::Existential(n) => Type::TypeVariable(*n + max),
            Type::Forall(v, t2) => Type::Forall(*v, t2.clone()),
            Type::Function(t1, t2) => {
                Type::Function(Box::new(t1.exist_to_tv(max)), Box::new(t2.exist_to_tv(max)))
            }
            _ => self.clone(),
        }
    }

    fn change_tv(&self, old: usize, new: usize) -> Self {
        match self {
            Type::TypeVariable(n) => {
                if *n == old {
                    Type::TypeVariable(new)
                } else {
                    self.clone()
                }
            }
            Type::Forall(v, t2) => {
                let v = if *v == old { new } else { *v };
                Type::Forall(v, Box::new(t2.change_tv(old, new)))
            }
            Type::Function(t1, t2) => Type::Function(
                Box::new(t1.change_tv(old, new)),
                Box::new(t2.change_tv(old, new)),
            ),
            _ => self.clone(),
        }
    }

    fn settle_tvs(&self) -> Self {
        let mut new_self = self.clone();
        for (new, old) in self.ordered_tvs().into_iter().enumerate() {
            new_self = new_self.change_tv(old, new);
        }
        new_self
    }

    pub fn forall_ify(&self) -> Self {
        let max_tv = self.max_type_var();
        let existentials: Vec<usize> = self.ordered_existentials().into_iter().collect();
        let new_self = self.exist_to_tv(max_tv);
        Type::fa(existentials, new_self)
    }

    pub fn is_monotype(&self) -> bool {
        match self {
            Type::Function(t1, t2) => t1.is_monotype() && t2.is_monotype(),
            Self::Forall(_, _) => false,
            _ => true,
        }
    }

    pub fn is_concrete(&self) -> bool {
        match self {
            Type::Primitive(_) => true,
            Type::Function(t1, t2) => t1.is_concrete() && t2.is_concrete(),
            _ => false,
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
            Type::Function(t1, t2) => std::cmp::max(t1.max_type_var(), t2.max_type_var()),
            Type::TypeVariable(n) => *n,
            _ => 0,
        }
    }

    fn add_to_type_vars(&self, increment: usize) -> Self {
        match self {
            Type::Function(t1, t2) => Type::Function(
                Box::new(t1.add_to_type_vars(increment)),
                Box::new(t2.add_to_type_vars(increment)),
            ),
            Type::TypeVariable(n) => Type::TypeVariable(n + increment),
            t => t.clone(),
        }
    }

    /// Make sure that t1 and t2 dont have overlapping type variables
    pub fn ensure_different_type_params(t1: Type, t2: Type) -> (Type, Type) {
        let t2 = t2.add_to_type_vars(t1.max_type_var() + 1);
        (t1, t2)
    }

    pub fn get_arity(&self) -> usize {
        match self {
            Type::Function(_, t) => 1 + t.get_arity(),
            _ => 0,
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
            Type::Existential(n) => {
                format!(
                    "E{}",
                    Type::TypeVariable(*n).to_string_internal(full_braces)
                )
            }
            Type::Unit => "1".to_string(),
            Type::Forall(n, t) => {
                format!(
                    "∀{}. {}",
                    Type::g(*n).to_string_internal(full_braces),
                    t.to_string_internal(full_braces)
                )
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_internal(false))
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
