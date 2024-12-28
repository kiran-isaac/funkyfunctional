use std::collections::HashMap;

use crate::{inbuilts::get_default_inbuilt_type_map, ASTNode, ASTNodeType, AST};

use super::{Type, TypeError};

pub struct TypeChecker {
    type_map: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            type_map: get_default_inbuilt_type_map(),
        }
    }

    fn type_error(&self, msg: String, node: &ASTNode) -> TypeError {
        return TypeError {
            e: msg,
            col: node.col,
            line: node.line,
        };
    }

    fn check_expression_expecting_type(
        &self,
        ast: &AST,
        module: usize,
        exp: usize,
        expected: &Type,
    ) -> Result<(), TypeError> {
        let found_type = match ast.get(exp).t {
            ASTNodeType::Identifier => {
                let id_name = ast.get(exp).get_value();

                let mut id_type: Option<Type> = None;
                if let Some(t) = self.type_map.get(&id_name) {
                    id_type = Some(t.clone());
                }

                if id_type.is_none() {
                    unreachable!("Cannot get type of ID. Should not be possible, bound checker must have bug")
                }

                id_type.unwrap()
            }
            ASTNodeType::Literal => {
                ast.get(exp).get_lit_type()

            }
            _ => unimplemented!()
        };

        if &found_type != expected {
            return Err(self.type_error(
                format!(
                    "Expected type {}, got type {}",
                    found_type.to_string(),
                    expected.to_string()
                ),
                ast.get(exp),
            ))
        }
        Ok(())
    }

    fn check_assign(
        &mut self,
        ast: &AST,
        assign: usize,
        module: usize,
        name: String,
    ) -> Result<(), TypeError> {
        let proclaimed_type = match &ast.get(assign).type_assignment {
            None => {
                return Err(self.type_error(
                    format!("Assignment without associated type assignment {}", name),
                    ast.get(assign),
                ))
            }
            Some(t) => t.clone(),
        };


        let expr = ast.get_assign_exp(assign);
        self.check_expression_expecting_type(ast, module, expr, &proclaimed_type)?;

        self.type_map.insert(name.clone(), proclaimed_type.clone());
        Ok(())
    }

    pub fn check_module(&mut self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
            self.check_assign(ast, assign, module, name)?;
        }
        Ok(())
    }
}
