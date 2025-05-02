use crate::{ASTNodeType, AST};
use std::collections::HashMap;

pub enum PatternMatchResult {
    Refute,
    // The bindings if successful
    Sucess(HashMap<String, usize>),
    Unknown,
}

use PatternMatchResult::*;

/// Get if pattern is matched, and returns bindings
pub fn pattern_match(ast: &AST, expr: usize, pattern: usize) -> PatternMatchResult {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);
    
    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);
    #[cfg(debug_assertions)]
    let _pat_str = ast.to_string_sugar(pattern, false);
    
    if pattern_n.t == ASTNodeType::Identifier {
        let pat_first_char = pattern_n.get_value().chars().nth(0).unwrap();
        match pat_first_char {
            // Wildcard
            'a'..='z' => {
                let mut map: HashMap<String, usize> = HashMap::new();
                map.insert(pattern_n.get_value(), expr);
                return Sucess(map);
            }
            // Non binding wildcard
            '_' => return Sucess(HashMap::new()),

            // Constructor
            'A'..='Z' => match expr_n.t {
                ASTNodeType::Identifier => {
                    let expr_first_char = expr_n.get_value().chars().nth(0).unwrap();
                    match expr_first_char {
                        'A'..='Z' => {
                            // If its another constructor, then we can refute the pattern
                            return if expr_n.get_value() == pattern_n.get_value() {
                                Sucess(HashMap::new())
                            } else {
                                Refute
                            }
                        },
                        // If its not a constructor we cant resolve the pattern
                        _ => return Unknown,
                    }
                }
                ASTNodeType::Application => {
                    let head_n = ast.get(ast.get_app_head(expr));
                    // We can only refute if the head of our application is a constructor as that means
                    // it will definitely not eval to this pattern  
                    if head_n.is_constructor() {
                        return Refute;
                    } else {
                        return Unknown;
                    }
                },
                ASTNodeType::Literal | ASTNodeType::Pair => return Refute,
                ASTNodeType::Abstraction | ASTNodeType::Match  => return Unknown,
                _ => unreachable!("Not an expression")
            },
            _ => unreachable!(),
        }
    }

    match (expr_n.t, pattern_n.t) {
        (ASTNodeType::Application, ASTNodeType::Application) => {
            let lhs = pattern_match(ast, ast.get_func(expr), ast.get_func(pattern));
            let rhs = pattern_match(ast, ast.get_arg(expr), ast.get_arg(pattern));
            match (lhs, rhs) {
                (Sucess(mut lhs), Sucess(rhs)) => {
                    lhs.extend(rhs);
                    Sucess(lhs)
                }
                (Unknown, _) | (_, Unknown) => Unknown,
                (Refute, _) | (_, Refute) => Refute,
            }
        }
        (_, ASTNodeType::Application) => Unknown,
        (ASTNodeType::Pair, ASTNodeType::Pair) => {
            let lhs = pattern_match(ast, ast.get_first(expr), ast.get_first(pattern));
            let rhs = pattern_match(ast, ast.get_second(expr), ast.get_second(pattern));
            match (lhs, rhs) {
                (Sucess(mut lhs), Sucess(rhs)) => {
                    lhs.extend(rhs);
                    Sucess(lhs)
                }
                (Unknown, _) | (_, Unknown) => Unknown,
                (Refute, _) | (_, Refute) => Refute,
            }
        }
        (ASTNodeType::Application, ASTNodeType::Pair) => {
            // If the head is a constructor then refute
            if ast.get(ast.get_app_head(expr)).is_constructor() {
                Refute
            } else {
                Unknown
            }
        }
        (ASTNodeType::Literal, ASTNodeType::Pair) => Refute,
        (ASTNodeType::Literal, ASTNodeType::Literal) => {
            if expr_n.get_value() == pattern_n.get_value() {
                Sucess(HashMap::new())
            } else {
                Refute
            }
        }
        _ => Unknown,
    }
}
