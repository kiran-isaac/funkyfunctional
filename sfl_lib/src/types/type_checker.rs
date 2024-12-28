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

    fn type_error(&self, msg: String, ast: &AST, node: usize) -> TypeError {
        let n = ast.get(node);
        return TypeError {
            e: format!("{}\n{}", msg, ast.to_string(node)),
            col: n.col,
            line: n.line,
        };
    }

    fn derive_expr_type(&mut self, ast: &AST, exp: usize) -> Result<Type, TypeError> {
        #[cfg(debug_assertions)]
        let _exp_str = ast.to_string(exp);
        
        match ast.get(exp).t {
            ASTNodeType::Literal => Ok(ast.get(exp).get_lit_type()),
            ASTNodeType::Identifier => {
                let name = ast.get(exp).get_value().clone();
                match self.type_map.get(&name) {
                    Some(t) => Ok(t.clone()),
                    None => unreachable!("derive_expr_type failiure: cannot get id type"),
                }
            }
            ASTNodeType::Application => {
                let f = ast.get_func(exp);
                let x = ast.get_arg(exp);

                let x_type = self.derive_expr_type(ast, x)?;

                #[cfg(debug_assertions)]
                let _x_type_str = x_type.to_string();

                match ast.get(f).t {
                    ASTNodeType::Identifier => {
                        let id_name = ast.get(f).get_value();

                        let mut id_type: Option<Type> = None;
                        if let Some(t) = self.type_map.get(&id_name) {
                            id_type = Some(t.clone());
                        }
                        let id_type = id_type.unwrap();

                        match id_type {
                            Type::Primitive(_) => Err(self.type_error(
                                format!(
                                    "Expected function type {} -> A, got a primitive type",
                                    x_type.to_string(),
                                ),
                                ast, exp,
                            )),
                            Type::Function(f_input_type, f_output_type) => {
                                if f_input_type.as_ref() != &x_type {
                                    Err(self.type_error(
                                        format!(
                                            "Expected function type {} -> A, got a primitive type",
                                            x_type.to_string(),
                                        ),
                                        ast, exp,
                                    ))
                                } else {
                                    Ok(f_output_type.as_ref().clone())
                                }
                            }
                        }
                    }
                    ASTNodeType::Application => {
                        let f_type = self.derive_expr_type(ast, ast.get_func(exp))?;

                        match f_type {
                            Type::Primitive(_) => Err(self.type_error(
                                format!(
                                    "Expected function type {} -> A, got a primitive type",
                                    x_type.to_string(),
                                ),
                                ast, exp,
                            )),
                            Type::Function(f_input_type, f_output_type) => {
                                if f_input_type.as_ref() != &x_type {
                                    Err(self.type_error(
                                        format!(
                                            "Expected function type {} -> A, got a primitive type",
                                            x_type.to_string(),
                                        ),
                                        ast, exp,
                                    ))
                                } else {
                                    Ok(f_output_type.as_ref().clone())
                                }
                            }
                        }
                    }
                    _ => Err(self.type_error(
                        format!("Expected function type, got {:?}", ast.get(f).t),
                        ast,f,
                    )),
                }
            }
            _ => unreachable!("Got non expr node"),
        }
    }

    fn check_expression_expecting_type(
        &mut self,
        ast: &AST,
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
                match expected {
                    Type::Primitive(_) => {
                        return Err(self.type_error(
                            format!(
                                "Expected function type {}, got a primitive type",
                                expected.to_string(),
                            ),
                            ast, exp,
                        ));
                    }
                    Type::Function(expected_f, expected_x) => {
                        let f_name = ast.get(ast.get_abstr_var(exp)).get_value();
                        self.type_map
                            .insert(f_name.clone(), expected_f.as_ref().clone());

                        self.check_expression_expecting_type(
                            ast,
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
                let x = ast.get_arg(exp);

                let f_type = self.derive_expr_type(ast, f)?;
                let x_type = self.derive_expr_type(ast, x)?;

                #[cfg(debug_assertions)]
                let _f_type_str = f_type.to_string();
                #[cfg(debug_assertions)]
                let _x_type_str = x_type.to_string();

                match f_type {
                    Type::Primitive(_) => {
                        return Err(self.type_error(
                            format!(
                                "Expected function type {}, got a primitive type",
                                expected.to_string(),
                            ),
                            ast,f,
                        ));
                    }
                    Type::Function(f_input_type, f_output_type) => {
                        if f_input_type.as_ref() != &x_type {
                            return Err(self.type_error(
                                format!(
                                    "Expected function type {} -> A, got a primitive type",
                                    x_type.to_string(),
                                ),
                                ast, f,
                            ));
                        } else {
                            f_output_type.as_ref().clone()
                        }
                    }
                }
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
                ast, exp,
            ));
        }
        Ok(())
    }

    fn check_assign(
        &mut self,
        ast: &AST,
        assign: usize,
        name: String,
    ) -> Result<(), TypeError> {
        let proclaimed_type = match &ast.get(assign).type_assignment {
            None => {
                return Err(self.type_error(
                    format!("Assignment without associated type assignment {}", name),
                    ast, assign,
                ))
            }
            Some(t) => t.clone(),
        };

        #[cfg(debug_assertions)]
        let _proclaimed_type_str = proclaimed_type.to_string();

        let expr = ast.get_assign_exp(assign);
        self.check_expression_expecting_type(ast, expr, &proclaimed_type)?;

        Ok(())
    }

    pub fn check_module(&mut self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
            let proclaimed_type = match &ast.get(assign).type_assignment {
                None => {
                    return Err(self.type_error(
                        format!("Assignment without associated type assignment {}", name),
                        ast, assign,
                    ))
                }
                Some(t) => t.clone(),
            };

            self.type_map.insert(name.clone(), proclaimed_type);
        }

        for (name, assign) in ast.get_assigns_map(module) {
            self.check_assign(ast, assign, name)?;
        }

        Ok(())
    }
}
