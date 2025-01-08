use std::collections::HashMap;

use crate::{functions::LabelTable, ASTNodeType, AST};

use super::{Type, TypeError};

pub struct TypeChecker {
    context: LabelTable,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context: LabelTable::new(),
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

    fn type_eq(&mut self, t1: &Type, t2: &Type) -> Result<(), String> {
        if t1.is_concrete() && t2.is_concrete() {
            if t1.concrete_eq(t2) {
                Ok(())
            } else {
                Err(format!("Cannot match type {} and {}", t1.to_string(), t2.to_string()))
            }
        } else {
            Ok(())
        }
    }

    fn synthesize_expression_type(
        &mut self,
        ast: &AST,
        expr: usize,
    ) -> Result<Type, TypeError> {
        #[cfg(debug_assertions)]
        let _exp_str = ast.to_string(expr);

        let node = ast.get(expr);
        match node.t {
            ASTNodeType::Identifier => {
                let name = node.get_value();

                match self.context.get_type(&name) {
                    None => {
                        Err(self.type_error(
                            format!("Unknown identifier {}", name),
                            ast,
                            expr,
                        ))
                    }
                    Some(t) => Ok(t.clone())
                }
            }
            ASTNodeType::Literal => {
                Ok(ast.get(expr).get_lit_type())
            }
            // Arrow introduction
            ASTNodeType::Abstraction => {
                let abst_var = ast.get(ast.get_abstr_var(expr));
                let abst_var_name = abst_var.get_value();

                let abst_var_type = match &abst_var.type_assignment {
                    Some(t) => t.clone(),
                    None => Type::g(0)
                };

                self.context.add(abst_var_name.clone(), abst_var_type);

                let abst_expr = ast.get_abstr_exp(expr);
                let abst_expr_type = self.synthesize_expression_type(ast, abst_expr)?;

                self.context.remove(&abst_var_name);

                // The expression may update the type of the type var, for instance (\x.x 1) would result in
                // (Int -> a) -> a
                let updated_var_type = self.context.get_type_map().get(&abst_var_name).unwrap().clone();
                Ok(Type::f(updated_var_type, abst_expr_type))
            }
            _ => unreachable!("Invalid synthesis call")
        }
    }

    fn check_expression_type(
        &mut self,
        ast: &AST,
        expr: usize,
        expected: &Type,
    ) -> Result<(), TypeError> {
        #[cfg(debug_assertions)]
        let _pattern_str = expected.to_string();
        #[cfg(debug_assertions)]
        let _exp_str = ast.to_string(expr);

        let node = ast.get(expr);
        match node.t {
            // Eq
            ASTNodeType::Identifier | ASTNodeType::Literal => {
                let synth_type = self.synthesize_expression_type(ast, expr)?;

                if expected == synth_type {
                    return Ok(())
                } else {
                    return Err(self.type_error(format!("Expected type {}, got type {}", expected.to_string(), ), ast, node))
                }
            },
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
        self.check_expression_type(ast, expr, &proclaimed_type)?;

        Ok(())
    }

    pub fn check_module(&mut self, ast: &AST, module: usize) -> Result<&LabelTable, TypeError> {
        self.context.consume_from_module(ast, module)?;

        for (name, assign) in ast.get_assigns_map(module) {
            self.check_assign(ast, assign, name)?;
        }

        Ok(&self.context)
    }
}
