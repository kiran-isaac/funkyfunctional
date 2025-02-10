use crate::{ASTNode, ASTNodeType, AST};
use std::collections::HashMap;

/// Get if pattern is matched, and returns bindings
pub fn pattern_match(ast: &AST, expr: usize, pattern: usize) -> Option<HashMap<String, usize>> {
    let expr_n = ast.get(expr);
    let pattern_n = ast.get(pattern);

    if pattern_n.t == ASTNodeType::Identifier {
        let first_char = pattern_n.get_value().chars().nth(0)?;
        match first_char {
            'a'..='z' => {
                let mut map: HashMap<String, usize> = HashMap::new();
                map.insert(pattern_n.get_value(), expr);
                return Some(map);
            }
            '_' => return Some(HashMap::new()),
            'A'..='Z' => match expr_n.t {
                ASTNodeType::Identifier => {
                    if expr_n.get_value() == pattern_n.get_value() {
                        return Some(HashMap::new());
                    }
                }
                _ => {}
            },
            _ => unreachable!(),
        }
    }

    match (expr_n.t.clone(), pattern_n.t.clone()) {
        (ASTNodeType::Application, ASTNodeType::Application) => {
            let lhs = pattern_match(ast, ast.get_func(expr), ast.get_func(pattern));
            let rhs = pattern_match(ast, ast.get_arg(expr), ast.get_arg(pattern));
            if lhs.is_none() || rhs.is_none() {
                None
            } else {
                let mut lhs = lhs.unwrap();
                lhs.extend(rhs.unwrap());
                Some(lhs)
            }
        }
        (ASTNodeType::Pair, ASTNodeType::Pair) => {
            let lhs = pattern_match(ast, ast.get_first(expr), ast.get_first(pattern));
            let rhs = pattern_match(ast, ast.get_second(expr), ast.get_second(pattern));
            if lhs.is_none() || rhs.is_none() {
                None
            } else {
                let mut lhs = lhs.unwrap();
                lhs.extend(rhs.unwrap());
                Some(lhs)
            }
        }
        (ASTNodeType::Literal, ASTNodeType::Literal) => {
            if expr_n.get_lit_type() != pattern_n.get_lit_type() {
                panic!("Not matching lit types, type checking must have failed")
            }

            if expr_n.get_value() == pattern_n.get_value() {
                Some(HashMap::new())
            } else {
                None
            }
        }
        _ => None,
    }
}
