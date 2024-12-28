use std::collections::HashMap;

use crate::{inbuilts::get_default_inbuilt_type_map, ASTNode, ASTNodeType, AST};

use super::{Primitive, Type, TypeError};

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
        &mut self,
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
            ASTNodeType::Literal => ast.get(exp).get_lit_type(),
            ASTNodeType::Abstraction => {
                // Check function type is expected
                if let Type::Primitive(_) = expected {}

                match expected {
                    Type::Primitive(_) => {
                        return Err(self.type_error(
                            format!(
                                "Expected primitive type {}, got a function type",
                                expected.to_string(),
                            ),
                            ast.get(exp),
                        ));
                    }
                    Type::Function(expected_f, expected_x) => {
                        let f_name = ast.get(ast.get_abstr_var(exp)).get_value();
                        self.type_map.insert(f_name, expected_f.as_ref().clone());

                        self.check_expression_expecting_type(
                            ast,
                            module,
                            ast.get_abstr_exp(exp),
                            expected_x.as_ref(),
                        )?;

                        self.type_map.remove(&f_name);

                        return Ok(());
                    }
                }
            }
            ASTNodeType::Application => {
                let f = ast.get_func(exp);
                let x = ast.get_func(exp);
            }
            _ => unimplemented!(),
        };

        if &found_type != expected {
            return Err(self.type_error(
                format!(
                    "Expected type {}, got type {}",
                    expected.to_string(),
                    found_type.to_string(),
                ),
                ast.get(exp),
            ));
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
