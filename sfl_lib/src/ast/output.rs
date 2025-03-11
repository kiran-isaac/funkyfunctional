use super::*;

impl AST {
    pub fn to_string_sugar(&self, node: usize, show_assigned_types: bool) -> String {
        let n = self.get(node);
        match n.t {
            ASTNodeType::Identifier => match &n.type_assignment {
                Some(t) => format!("{} :: {}", n.get_value(), t.to_string()),
                None => n.get_value(),
            },
            ASTNodeType::Literal => {
                format!("{}", n.get_value())
            }
            ASTNodeType::Application => {
                let func = self.get_func(node);
                let arg = self.get_arg(node);
                let func_str = self.to_string_sugar(func, show_assigned_types);
                let arg_str = self.to_string_sugar(arg, show_assigned_types);

                if n.dollar_app {
                    return format!("{} $ {}", func_str, arg_str);
                }
                // If the func is an abstraction, wrap it in parens
                let func_str = match self.get(func).t {
                    ASTNodeType::Abstraction => format!("({})", func_str),
                    _ => func_str,
                };
                // If the argument is an application, wrap it in parens
                let arg_str = match self.get(arg).t {
                    ASTNodeType::Application | ASTNodeType::Abstraction => format!("({})", arg_str),
                    _ => arg_str,
                };

                if let Some(tk) = &self.get(func).info {
                    if tk.is_infix_id() {
                        return format!("{} {}", arg_str, func_str);
                    }
                }

                format!("{} {}", func_str, arg_str)
            }
            ASTNodeType::Match => {
                let mut s = "match ".to_string();
                let unpack_pattern = self.get_match_unpack_pattern(node);
                s.push('(');
                s.push_str(&self.to_string_sugar(unpack_pattern, false));
                s.push(')');
                s.push(' ');
                s.push('{');
                s.push('\n');
                for (pat, exp) in self.get_match_cases(node) {
                    s.push_str("  | ");
                    s.push_str(&self.to_string_sugar(pat, false));
                    s.push_str(" -> ");
                    s.push_str(&self.to_string_sugar(exp, show_assigned_types));
                    s.push('\n');
                }
                s.push('}');
                s
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let var_name = id.get_value();
                let mut exp = self.to_string_sugar(self.get_assign_exp(node), show_assigned_types);

                let mut fancy_syntax_abst_vars = "".to_string();
                let mut ass_abst = self.get_assign_exp(node);

                while self.get(ass_abst).fancy_assign_abst_syntax {
                    assert_eq!(self.get(ass_abst).t, ASTNodeType::Abstraction);
                    fancy_syntax_abst_vars += " ";
                    fancy_syntax_abst_vars += self
                        .to_string_sugar(self.get_abstr_var(ass_abst), show_assigned_types)
                        .as_str();
                    exp = self.to_string_sugar(self.get_abstr_expr(ass_abst), show_assigned_types);
                    ass_abst = self.get_abstr_expr(ass_abst);
                }

                let type_str = if show_assigned_types {
                    if let Some(ass_type) = &self.get(node).type_assignment {
                        var_name.clone() + " :: " + ass_type.to_string().as_str() + "\n"
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };

                format!(
                    "{}{}{} = {}",
                    type_str,
                    id.get_value(),
                    fancy_syntax_abst_vars,
                    exp
                )
            }
            ASTNodeType::Module => {
                let mut s = String::new();
                for c in &n.children {
                    s.push_str(&self.to_string_sugar(*c, show_assigned_types));
                    s.push_str("\n");
                }

                s.trim().to_string()
            }
            ASTNodeType::Abstraction => {
                let expr_str = self.to_string_sugar(n.children[1], show_assigned_types);
                let var_str = self.to_string_sugar(n.children[0], show_assigned_types);

                let mut res = "\\".to_string();
                res.push_str(&var_str);
                res.push_str(". ");
                res.push_str(&expr_str);
                res
            }
            ASTNodeType::Pair => {
                let a = self.to_string_sugar(self.get_first(node), show_assigned_types);
                let b = self.to_string_sugar(self.get_second(node), show_assigned_types);
                format!("({}, {})", a, b)
            }
        }
    }

    pub fn type_assigns_to_string(&self, module: usize) -> String {
        let n = self.get(module);
        let mut s = String::new();
        for c in &n.children {
            let type_assign = self.get(*c).type_assignment.clone().unwrap();
            let assign_name = self.get_assignee(*c);
            s.push_str(format!("{} :: {}\n", assign_name, type_assign.to_string()).as_str());
        }

        s.trim().to_string()
    }

    pub fn to_string_desugar_and_type(&self, node: usize) -> String {
        let n = self.get(node);
        match n.t {
            ASTNodeType::Identifier => match &n.type_assignment {
                Some(t) => format!("{} :: {}", n.get_value(), t.to_string()),
                None => n.get_value(),
            },
            ASTNodeType::Literal => {
                format!("{}", n.get_value())
            }
            ASTNodeType::Application => {
                let func = self.get_func(node);
                let arg = self.get_arg(node);

                let func_str = self.to_string_desugar_and_type(func);

                // If the func is an abstraction, wrap it in parens
                let func_str = match self.get(func).t {
                    ASTNodeType::Abstraction => format!("({})", func_str),
                    _ => func_str,
                };

                let arg_str = self.to_string_desugar_and_type(arg);
                // If the argument is an application, wrap it in parens
                let arg_str = match self.get(arg).t {
                    ASTNodeType::Application | ASTNodeType::Abstraction => format!("({})", arg_str),
                    _ => arg_str,
                };

                format!("{} {}", func_str, arg_str)
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let var_name = id.get_value();
                let exp = self.to_string_desugar_and_type(self.get_assign_exp(node));

                let type_str = if let Some(ass_type) = &self.get(node).type_assignment {
                    var_name.clone() + " :: " + ass_type.to_string().as_str() + "\n"
                } else {
                    "".to_string()
                };

                format!("{}{} = {}", type_str, &var_name, exp)
            }
            ASTNodeType::Module => {
                let mut s = String::new();
                for c in &n.children {
                    s.push_str(&self.to_string_desugar_and_type(*c));
                    s.push_str("\n");
                }

                s.trim().to_string()
            }
            ASTNodeType::Match => {
                let mut s = "match ".to_string();
                let unpack_pattern = self.get_match_unpack_pattern(node);
                s.push_str(&self.to_string_desugar_and_type(unpack_pattern));
                for (pat, exp) in self.get_match_cases(node) {
                    s.push_str(" | ");
                    s.push_str(&self.to_string_desugar_and_type(pat));
                    s.push_str(" -> ");
                    s.push_str(&self.to_string_desugar_and_type(exp));
                    s.push('\n');
                }
                s.pop();
                s
            }
            ASTNodeType::Abstraction => {
                let expr_str = self.to_string_desugar_and_type(n.children[1]);
                let var_str = self.to_string_desugar_and_type(n.children[0]);

                let mut res = "\\".to_string();
                res.push_str(&var_str);
                res.push_str(" . ");
                res.push_str(&expr_str);
                res
            }
            ASTNodeType::Pair => {
                let a = self.to_string_desugar_and_type(self.get_first(node));
                let b = self.to_string_desugar_and_type(self.get_second(node));
                format!("({}, {})", a, b)
            }
        }
    }
}
