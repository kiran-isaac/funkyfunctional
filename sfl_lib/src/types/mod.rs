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
    Char,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Function(Box<Type>, Box<Type>),
    Generic(usize),
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
        Type::Generic(usize)
    }

    fn is_concrete(&self) -> bool {
        match self {
            Type::Primitive(_) => true,
            Type::Function(t1, t2) => t1.is_concrete() && t2.is_concrete(),
            Type::Generic(_) => false,
        }
    }

    fn concrete_eq(&self, other: &Type) -> bool {
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
            (Type::Generic(_), _) | (_, Type::Generic(_)) => unreachable!(),
            _ => false,
        }
    }

    fn fill_type_pattern_recurse(
        &self,
        other: &Type,
        generic_map: &mut HashMap<usize, Type>,
    ) -> Result<Type, String> {
        #[cfg(debug_assertions)]
        let _pattern_str = self.to_string();
        #[cfg(debug_assertions)]
        let _other_str = other.to_string();
        #[cfg(debug_assertions)]
        let mut _generic_map_str = format!("{:?}", generic_map);

        match (self, other) {
            (Type::Function(f1, x1), Type::Function(f2, x2)) => {
                let f_type = f1.fill_type_pattern_recurse(f2, generic_map)?;
                #[cfg(debug_assertions)]
                {
                    _generic_map_str = format!("{:?}", generic_map);
                }

                let x_type = x1.fill_type_pattern_recurse(x2, generic_map)?;
                #[cfg(debug_assertions)]
                {
                    _generic_map_str = format!("{:?}", generic_map);
                }
                Ok(Type::Function(Box::new(f_type), Box::new(x_type)))
            }
            (Type::Primitive(p1), Type::Primitive(p2)) => {
                if *p1 != *p2 {
                    Err(format!("Failed to match types {} and {}", self.to_string(), self.to_string()))
                } else {
                    Ok(self.clone())
                }
            }
            (Type::Generic(g), _) => {
                if other.is_concrete() {
                    if let Some(t) = generic_map.get(g) {
                        if t.concrete_eq(other) {
                            Ok(t.clone())
                        } else {
                            Err(format!(
                                "Generic type {} cannot match both {} and {}",
                                self.to_string(),
                                t.to_string(),
                                other.to_string()
                            ))
                        }
                    } else {
                        generic_map.insert(*g, other.clone());
                        Ok(other.clone())
                    }
                } else {
                    if let Some(t) = generic_map.get(g) {
                        Ok(t.clone())
                    } else {
                        Err(format!(
                            "Insufficient type information to reconsile {} and {}",
                            self.to_string(),
                            other.to_string()
                        ))
                    }
                }
            }
            (_, Type::Generic(g)) => {
                if self.is_concrete() {
                    if let Some(t) = generic_map.get(g) {
                        if t.concrete_eq(self) {
                            Ok(t.clone())
                        } else {
                            Err(format!(
                                "Generic type {} cannot match both {} and {}",
                                other.to_string(),
                                t.to_string(),
                                self.to_string()
                            ))
                        }
                    } else {
                        generic_map.insert(*g, self.clone());
                        Ok(self.clone())
                    }
                } else {
                    if let Some(t) = generic_map.get(g) {
                        Ok(t.clone())
                    } else {
                        Err(format!(
                            "Insufficient type information to reconsile {} and {}",
                            self.to_string(),
                            self.to_string()
                        ))
                    }
                }
            }
            _ => Err(format!("Failed to match types {} and {}", self.to_string(), other.to_string())),
        }
    }

    /// If the pattern is a -> b -> a and the other type is Int -> Float -> a, this function
    /// extrapolates the type of a to be Int and returns the type Int -> Float -> Int
    fn fill_pattern(&self, other: &Type) -> Result<Type, String> {
        let t = self.fill_type_pattern_recurse(other, &mut HashMap::new())?;
        #[cfg(debug_assertions)]
        assert!(t.is_concrete());

        Ok(t)
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
            Type::Generic(n) => {
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

        let t6 = Type::Generic(0);
        assert_eq!(t6.to_string(), "a");

        println!("{:?}", t6);

        let t6 = Type::Generic(26);
        assert_eq!(t6.to_string(), "aa");

        let t6 = Type::Generic(27);
        assert_eq!(t6.to_string(), "ab");

        let t6 = Type::Generic(28);
        assert_eq!(t6.to_string(), "ac");

        let t6 = Type::Generic(26 * 2);
        assert_eq!(t6.to_string(), "ba");
    }
}
