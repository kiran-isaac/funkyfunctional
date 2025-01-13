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
    TypeVariable(String),
    Existential(usize),
    Forall(String, Box<Type>),
    Product(Box<Type>, Box<Type>),
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

    pub fn tv(name: String) -> Type {
        Type::TypeVariable(name)
    }

    pub fn pr(t1: Type, t2: Type) -> Type {
        Type::Product(Box::new(t1), Box::new(t2))
    }

    pub fn fa(forall: Vec<String>, t: Self) -> Self {
        let mut t = t;
        for i in forall.into_iter().rev() {
            t = Type::Forall(i, Box::new(t));
        }
        t
    }

    pub fn contains_existential(&self, ex: usize) -> bool {
        match self {
            Type::Primitive(_) => false,
            Type::Function(t1, t2) | Type::Product(t1, t2) => {
                t1.contains_existential(ex) || t2.contains_existential(ex)
            }
            Type::TypeVariable(_) => false,
            Type::Existential(e) => *e == ex,
            Type::Forall(_, t) => t.contains_existential(ex),
            Type::Unit => false,
        }
    }

    pub fn substitute_type_variable(
        &self,
        var: &String,
        replacement: &Type,
    ) -> Result<Self, String> {
        match self {
            Type::Primitive(p) => Ok(Type::Primitive(*p)),
            Type::Function(t1, t2) => Ok(Type::Function(
                Box::new(t1.substitute_type_variable(var, replacement)?),
                Box::new(t2.substitute_type_variable(var, replacement)?),
            )),
            Type::TypeVariable(n) => {
                if n == var {
                    Ok(replacement.clone())
                } else {
                    Ok(Type::TypeVariable(n.clone()))
                }
            }
            Type::Forall(var2, t2) => {
                if var2 == var {
                    panic!("Duplicate forall")
                }
                Ok(Type::fa(
                    vec![var2.clone()],
                    t2.substitute_type_variable(var, replacement)?,
                ))
            }
            Type::Product(t1, t2) => {
                Ok(Type::pr(t1.substitute_type_variable(var, replacement)?, t2.substitute_type_variable(var, replacement)?))
            }
            _ => Ok(self.clone()),
        }
    }

    fn remove_duplicates<T: Eq + Hash + Clone>(ls: &Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        let mut new_vec: Vec<T> = Vec::new();
        for i in ls {
            if !seen.contains(i) {
                new_vec.push(i.clone());
                seen.insert(i);
            }
        }
        new_vec
    }

    fn ordered_existentials(&self) -> Vec<usize> {
        match &self {
            Type::Existential(n) => vec![*n],
            Type::Forall(_, t2) => t2.ordered_existentials(),
            Type::Function(t1, t2) | Type::Product(t1, t2) => {
                let mut t1 = t1.ordered_existentials();
                let t2 = t2.ordered_existentials();
                t1.extend(t2);
                Self::remove_duplicates(&t1)
            }
            _ => vec![],
        }
    }

    /// Convert all exists with id ext to TVs
    fn exist_to_tv(&self, ext: usize, str: &String) -> Self {
        match self {
            Type::Existential(n) => {
                if *n == ext {
                    Type::TypeVariable(str.to_string())
                } else {
                    self.clone()
                }
            }
            Type::Forall(v, t2) => Type::Forall(v.clone(), Box::new(t2.exist_to_tv(ext, str))),
            Type::Function(t1, t2) => {
                let lhs = t1.exist_to_tv(ext, str);
                let rhs = t2.exist_to_tv(ext, str);
                Type::Function(Box::new(lhs), Box::new(rhs))
            }
            Type::Product(t1, t2) => {
                let lhs = t1.exist_to_tv(ext, str);
                let rhs = t2.exist_to_tv(ext, str);
                Type::Product(Box::new(lhs), Box::new(rhs))
            }
            _ => self.clone(),
        }
    }

    pub fn forall_ify(&self) -> Self {
        let mut tv_set = self.get_tvs_set();
        let mut exsts = vec![];
        let mut new_self = self.clone();
        for (index, exst) in self.ordered_existentials().into_iter().enumerate() {
            let mut str = Self::num_identifier_to_str(index);
            let mut i = 0;
            while tv_set.contains(&str) {
                i += 1;
                str = Self::num_identifier_to_str(index + i)
            }
            tv_set.insert(str.clone());
            exsts.push(str.clone());
            new_self = new_self.exist_to_tv(exst, &str);
        }
        Type::fa(exsts, new_self)
    }

    pub fn is_monotype(&self) -> bool {
        match self {
            Type::Function(t1, t2) => t1.is_monotype() && t2.is_monotype(),
            Type::Product(t1, t2) => t1.is_monotype() && t2.is_monotype(),
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

    pub fn flatten(&self) -> Vec<Self> {
        match self {
            Type::Function(t1, t2) => {
                vec![t1.as_ref().clone()].into_iter().chain(t2.flatten().into_iter()).collect()
            }
            Type::Forall(_, t1) => {
                t1.flatten()
            }
            _ => vec![self.clone()],
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

    fn get_tvs_set(&self) -> HashSet<String> {
        match self {
            Type::Function(t1, t2) => {
                let mut t1 = t1.get_tvs_set();
                let t2 = t2.get_tvs_set();
                t1.extend(t2);
                t1
            }
            Type::Forall(str1, t1) => {
                let mut t1 = t1.get_tvs_set();
                t1.insert(str1.clone());
                t1
            }
            Type::TypeVariable(str) => HashSet::from_iter(vec![str.clone()]),
            _ => HashSet::new(),
        }
    }

    pub fn get_arity(&self) -> usize {
        match self {
            Type::Function(_, t) => 1 + t.get_arity(),
            _ => 0,
        }
    }

    fn num_identifier_to_str(n: usize) -> String {
        let mut s = String::new();
        let mut n = n;
        s.insert(0, (b'a' + (n % 26) as u8) as char);
        n /= 26;
        while n > 0 {
            s.insert(0, (b'a' + (n % 26 - 1) as u8) as char);
            n /= 26;
        }
        s
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
            Type::TypeVariable(n) => n.clone(),
            Type::Existential(n) => {
                format!("E{}", Self::num_identifier_to_str(*n))
            }
            Type::Unit => "1".to_string(),
            Type::Forall(n, t) => {
                format!("∀{}. {}", n, t.to_string_internal(full_braces))
            }
            Type::Product(t1, t2) => {
                format!(
                    "({}, {})",
                    t1.to_string_internal(full_braces),
                    t2.to_string_internal(full_braces)
                )
            }
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Type::Primitive(*self).to_string_internal(false))
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

        let t6 = Type::Existential(0);
        assert_eq!(t6.to_string(), "Ea");

        let t6 = Type::Existential(26);
        assert_eq!(t6.to_string(), "Eaa");

        let t6 = Type::Existential(27);
        assert_eq!(t6.to_string(), "Eab");

        let t6 = Type::Existential(28);
        assert_eq!(t6.to_string(), "Eac");

        let t6 = Type::Existential(26 * 2);
        assert_eq!(t6.to_string(), "Eba");
    }
}
