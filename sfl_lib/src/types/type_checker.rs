use std::collections::HashMap;

use crate::{inbuilts::get_default_inbuilt_type_map, ASTNode, AST};

use super::{Type, TypeError};

struct TypeChecker {
    inbuilt_type_map: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            inbuilt_type_map: get_default_inbuilt_type_map(),
        }
    }

    fn type_error(&self, msg: String, node: &ASTNode) -> TypeError {
        return TypeError {
            e: msg,
            col: node.col,
            line: node.line,
        };
    }

    // fn check_expression_expecting_type(&self, ast: &AST, exp: usize) -> Result<(), TypeError> {
    //     match ast.get(exp).t {
            
    //     }

    //     Ok(())
    // }

    pub fn check_module(&self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
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
        }
        Ok(())
    }
}
