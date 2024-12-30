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
            e: msg,
            col: n.col,
            line: n.line,
        };
    }

    fn check_expression(
        &mut self,
        ast: &AST,
        exp: usize,
        expected: &Type,
    ) -> Result<Type, TypeError> {
        #[cfg(debug_assertions)]
        let _pattern_str = expected.to_string();
        #[cfg(debug_assertions)]
        let _exp_str = ast.to_string(exp);

        let node = ast.get(exp);
        match node.t {
            ASTNodeType::Identifier => {
                let name = node.get_value();

                match self.type_map.get(&name) {
                    None => {
                        return Err(self.type_error(
                            format!("Unknown identifier {}", name),
                            ast,
                            exp,
                        ))
                    }
                    Some(t) => {
                        #[cfg(debug_assertions)]
                        let _t_str = t.to_string();
                        let filled = t.fill_pattern(expected);
                        match filled {
                            Ok(fill) => Ok(fill),
                            Err(e) => Err(self.type_error(
                                format!(
                                    "Failed to typecheck variable {}, could not reconcile type {} with type {}: {}",
                                    name, 
                                    expected.to_string(),
                                    t.to_string(),
                                    e
                                ),
                                ast,
                                exp,
                            )),
                        }
                    }
                }
            }
            ASTNodeType::Literal => {
                let lit_type = ast.get(exp).get_lit_type();
                let filled = expected.fill_pattern(&lit_type);
                match filled {
                    Ok(fill) => Ok(fill),
                    Err(_) => Err(self.type_error(
                        format!(
                            "Failed to match pattern {} with literal {}\n{:?}",
                            expected.to_string(),
                            lit_type.to_string(),
                            ast.to_string(exp)
                        ),
                        ast,
                        exp,
                    )),
                }
            }
            ASTNodeType::Application => {
                let f = ast.get_func(exp);
                let x = ast.get_arg(exp);

                #[cfg(debug_assertions)]
                let _f_str = ast.to_string(f);
                #[cfg(debug_assertions)]
                let _x_str = ast.to_string(x);

                let x_type = self.check_expression(ast, x, &Type::g(0))?;
                if !x_type.is_concrete() {
                    unimplemented!()
                }

                let f_pattern = Type::f(x_type.clone(), expected.clone());
                #[cfg(debug_assertions)]
                let _f_pattern_str = f_pattern.to_string();
                let f_type = self.check_expression(ast, f, &f_pattern)?;
                #[cfg(debug_assertions)]
                let _f_type_str = f_type.to_string();

                match f_type {
                    Type::Function(f_f_type, f_x_type) => {
                        #[cfg(debug_assertions)]
                        assert!(f_f_type.is_concrete() && f_f_type.concrete_eq(&x_type));

                        Ok(f_x_type.as_ref().clone())
                    }
                    _ => unimplemented!(),
                }
            }
            ASTNodeType::Abstraction => {
                match expected {
                    Type::Function(f, x) => {
                        let var = ast.get_abstr_var(exp);
                        let var_name = ast.get(var).get_value();
                        let var_type = f.as_ref().clone();

                        #[cfg(debug_assertions)]
                        let _var_type_str = var_type.to_string();

                        match var_type {
                            Type::Generic(_) => panic!("Generic type in function argument"),
                            _ => {}
                        }

                        // If there is a type assignment, make sure its correct
                        match &ast.get(var).type_assignment {
                            Some(t) => {
                                let filled = var_type.fill_pattern(t);
                                match filled {
                                    Ok(_) => {}
                                    Err(_) => {
                                        return Err(self.type_error(
                                            format!(
                                                "Failed to type check abstraction {}\nAbstraction variable is labled as having type {}, but it is used as if it has type {}\n",
                                                ast.to_string(exp),
                                                t.to_string(),
                                                var_type.to_string(),
                                            ),
                                            ast,
                                            exp,
                                        ))
                                    }
                                }
                            }
                            None => {}
                        }

                        #[cfg(debug_assertions)]
                        let _var_type_str = var_type.to_string();

                        self.type_map.insert(var_name.clone(), var_type.clone());
                        let abst_exp = ast.get_abstr_exp(exp);
                        let abst_exp_type = match self.check_expression(ast, abst_exp, x) {
                            Ok(t) => t,
                            Err(e) => {
                                return Err(self.type_error(
                                    format!("Failed to type check abstraction {}.\nThe abstraction's expression was expected to have type {}, but got this error while verifying the expression type:\n{}", ast.to_string(exp), x.to_string(), e.e),
                                    ast,
                                    exp,
                                ))
                            }
                        };

                        self.type_map.remove(&var_name);

                        #[cfg(debug_assertions)]
                        let _abst_exp_type_str = abst_exp_type.to_string();

                        Ok(Type::f(var_type, abst_exp_type))
                    }
                    Type::Generic(_) => {
                        let var = ast.get_abstr_var(exp);
                        let var_name = ast.get(var).get_value();
                        let var_type = ast.get(var).type_assignment.clone();

                        if var_type.is_none() {
                            return Err(self.type_error(
                                format!(
                                    "Abstraction {} needs type information for its variable {}",
                                    ast.to_string(exp),
                                    var_name,
                                ),
                                ast,
                                var,
                            ));
                        }

                        let var_type = var_type.unwrap();
                        #[cfg(debug_assertions)]
                        let _var_type_str = var_type.to_string();

                        self.type_map.insert(var_name.clone(), var_type.clone());
                        let abst_exp = ast.get_abstr_exp(exp);
                        let abst_exp_type =
                            self.check_expression(ast, abst_exp, &Type::Generic(0))?;
                        self.type_map.remove(&var_name);

                        #[cfg(debug_assertions)]
                        let _abst_exp_type_str = abst_exp_type.to_string();

                        Ok(Type::f(var_type, abst_exp_type))
                    }
                    Type::Primitive(_) => {
                        return Err(self.type_error(
                            format!("Expected function type, got {}", expected.to_string()),
                            ast,
                            exp,
                        ))
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    fn check_assign(&mut self, ast: &AST, assign: usize, name: String) -> Result<(), TypeError> {
        let proclaimed_type = match &ast.get(assign).type_assignment {
            None => {
                return Err(self.type_error(
                    format!("Assignment without associated type assignment {name}"),
                    ast,
                    assign,
                ))
            }
            Some(t) => t.clone(),
        };

        #[cfg(debug_assertions)]
        let _proclaimed_type_str = proclaimed_type.to_string();

        let expr = ast.get_assign_exp(assign);
        self.check_expression(ast, expr, &proclaimed_type)?;

        Ok(())
    }

    pub fn check_module(&mut self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
            let proclaimed_type = match &ast.get(assign).type_assignment {
                None => {
                    return Err(self.type_error(
                        format!("Assignment without associated type assignment {}", name),
                        ast,
                        assign,
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
