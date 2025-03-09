use super::*;

#[derive(Clone)]
pub enum ASTDiffElem {
    Similar(String),
    Different(String, String),
}

#[derive(Clone)]
pub struct ASTDiff {
    vec: Vec<ASTDiffElem>,
}

impl ASTDiff {
    fn new() -> Self {
        Self { vec: vec![] }
    }

    fn const_str(&mut self, str: &str) {
        self.str(str.to_string());
    }

    fn str(&mut self, str: String) {
        self.vec.push(ASTDiffElem::Similar(str))
    }

    fn diff(&mut self, str1: String, str2: String) {
        self.vec.push(ASTDiffElem::Different(str1, str2));
    }

    fn extend(&mut self, other: ASTDiff) {
        self.vec.extend(other.vec);
    }

    fn prepend(&mut self, elem: ASTDiffElem) {
        self.vec.insert(0, elem);
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn get(&self, index: usize) -> Option<&ASTDiffElem> {
        self.vec.get(index)
    }

    pub fn str_1(&self) -> String {
        let mut str = String::new();
        for elem in &self.vec {
            match elem {
                ASTDiffElem::Similar(str2) | ASTDiffElem::Different(str2, _) => str.push_str(str2),
            }
        }
        str
    }

    pub fn str_2(&self) -> String {
        let mut str = String::new();
        for elem in &self.vec {
            match elem {
                ASTDiffElem::Similar(str2) | ASTDiffElem::Different(_, str2) => str.push_str(str2),
            }
        }
        str
    }
}

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
                let mut func = node;
                let mut args = vec![];
                for _ in 0..3 {
                    match self.get(func).t {
                        ASTNodeType::Application => {
                            args.push(self.get_arg(func));
                            func = self.get_func(func);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                if args.len() == 3 {
                    match self.get(func).t {
                        ASTNodeType::Identifier => {
                            if self.get(func).get_value() == "if" {
                                return format!(
                                    "if {} then {} else {}",
                                    self.to_string_sugar(args[2], show_assigned_types),
                                    self.to_string_sugar(args[1], show_assigned_types),
                                    self.to_string_sugar(args[0], show_assigned_types)
                                );
                            }
                        }
                        _ => {}
                    }
                }

                let func = self.get_func(node);
                let arg = self.get_arg(node);
                let func_str = self.to_string_sugar(func, show_assigned_types);
                let arg_str = self.to_string_sugar(arg, show_assigned_types);

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

                if n.dollar_app {
                    format!("{} $ {}", func_str, arg_str)
                } else {
                    format!("{} {}", func_str, arg_str)
                }
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

    /// Generate the strings for old and new as a diff
    /// returns similarieies, and pairs of differences
    pub fn diff(old: &AST, new: &AST, expr1: usize, expr2: usize) -> ASTDiff {
        let n1 = old.get(expr1);
        let n2 = new.get(expr2);

        let mut diff = ASTDiff::new();

        // Expr eq is identical to str_eq im pretty sure, and they both require tree traversal

        match (&n1.t, &n2.t) {
            (ASTNodeType::Pair, ASTNodeType::Pair) => {
                diff.const_str("(");

                diff.extend(AST::diff(
                    old,
                    new,
                    old.get_first(expr1),
                    new.get_first(expr2),
                ));
                diff.const_str(",");
                diff.extend(AST::diff(
                    old,
                    new,
                    old.get_second(expr1),
                    new.get_second(expr2),
                ));
                diff.const_str(")");
            }
            (ASTNodeType::Abstraction, ASTNodeType::Abstraction) => {
                diff.const_str("\\");
                diff.extend(AST::diff(
                    old,
                    new,
                    old.get_abstr_var(expr1),
                    new.get_abstr_var(expr2),
                ));
                diff.const_str(". ");
                diff.extend(AST::diff(
                    old,
                    new,
                    old.get_abstr_expr(expr1),
                    new.get_abstr_expr(expr2),
                ));
            }
            (ASTNodeType::Match, ASTNodeType::Match) => {
                let old_cases = old.get_match_cases(expr1);
                let new_cases = new.get_match_cases(expr2);
                let mut cases_are_different = false;

                if old_cases.len() == new_cases.len() {
                    for ((old_case, _), (new_case, _)) in zip(&old_cases, &new_cases) {
                        if !AST::eq(old, new, *old_case, *new_case) {
                            cases_are_different = true;
                            break;
                        }
                    }
                }

                // If there are different number of cases, or cases are diffent, these are alltogether different match statements
                if old_cases.len() != new_cases.len() || cases_are_different {
                    let str_old = old.to_string_sugar(expr1, false);
                    let str_new = new.to_string_sugar(expr2, false);
                    #[cfg(debug_assertions)]
                    assert_ne!(str_new, str_old);
                    diff.diff(str_old, str_new);
                }

                diff.const_str("match (");
                diff.extend(AST::diff(old, new, old.get_match_unpack_pattern(expr1), new.get_match_unpack_pattern(expr2)));
                diff.const_str(") {\n");

                // We know the cases are the same by here
                for ((old_case, old_expr), (_, new_expr)) in zip(old_cases, new_cases) {
                    diff.const_str("  | ");
                    diff.str(old.to_string_sugar(old_case, false));
                    diff.const_str(" -> ");
                    diff.extend(AST::diff(old, new, old_expr, new_expr));
                    diff.const_str("\n");
                }
                diff.const_str("}");
            }

            (ASTNodeType::Application, ASTNodeType::Application) => {
                diff.const_str("");
                let mut func_diff = AST::diff(old, new, old.get_func(expr1), new.get_func(expr2));
                let mut arg_diff = AST::diff(old, new, old.get_arg(expr1), new.get_arg(expr2));

                let old_func = old.get(old.get_func(expr1));
                let new_func = new.get(new.get_func(expr2));
                let old_arg = old.get(old.get_arg(expr1));
                let new_arg = new.get(new.get_arg(expr2));

                let old_func_needs_brackets = old_func.t == ASTNodeType::Abstraction;
                let new_func_needs_brackets = new_func.t == ASTNodeType::Abstraction;
                let old_arg_needs_brackets =
                    old_arg.t == ASTNodeType::Abstraction || old_arg.t == ASTNodeType::Application;
                let new_arg_needs_brackets =
                    new_arg.t == ASTNodeType::Abstraction || old_arg.t == ASTNodeType::Application;

                match (old_func_needs_brackets, new_func_needs_brackets) {
                    (true, true) => {
                        func_diff.prepend(ASTDiffElem::Similar("(".to_string()));
                        func_diff.const_str(")");
                    }
                    (true, false) => {
                        func_diff.prepend(ASTDiffElem::Different("(".to_string(), "".to_string()));
                        func_diff.diff(")".to_string(), "".to_string());
                    }
                    (false, true) => {
                        func_diff.prepend(ASTDiffElem::Different("".to_string(), ")".to_string()));
                        func_diff.diff("".to_string(), ")".to_string());
                    }
                    (false, false) => {}
                }

                match (old_arg_needs_brackets, new_arg_needs_brackets) {
                    (true, true) => {
                        arg_diff.prepend(ASTDiffElem::Similar("(".to_string()));
                        arg_diff.const_str(")");
                    }
                    (true, false) => {
                        arg_diff.prepend(ASTDiffElem::Different("(".to_string(), "".to_string()));
                        arg_diff.diff(")".to_string(), "".to_string());
                    }
                    (false, true) => {
                        arg_diff.prepend(ASTDiffElem::Different("".to_string(), ")".to_string()));
                        arg_diff.diff("".to_string(), ")".to_string());
                    }
                    (false, false) => {}
                }

                let old_is_infix = match old_func.t {
                    ASTNodeType::Identifier => old_func.info.clone().unwrap().is_infix_id(),
                    _ => false,
                };
                let new_is_infix = match new_func.t {
                    ASTNodeType::Identifier => new_func.info.clone().unwrap().is_infix_id(),
                    _ => false,
                };

                match (old_is_infix, new_is_infix) {
                    (true, true) => {
                        diff.extend(arg_diff);
                        diff.extend(func_diff);
                    }
                    (false, false) => {
                        diff.extend(func_diff);
                        if n1.dollar_app && n2.dollar_app {
                            diff.const_str(" $ ");
                        } else {
                            diff.const_str(" ");
                        }
                        diff.extend(arg_diff);
                    }
                    _ => {
                        let str_old = old.to_string_sugar(expr1, false);
                        let str_new = new.to_string_sugar(expr2, false);
                        diff.diff(str_old, str_new);
                    }
                }
            }

            // Catchall, for ids that are different, lits that are different, or completely different structures
            (_, _) => {
                let str_old = old.to_string_sugar(expr1, false);
                let str_new = new.to_string_sugar(expr2, false);
                if &str_old == &str_new {
                    diff.str(str_old);
                } else {
                    diff.diff(str_old, str_new);
                }
            }
        }

        diff
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

#[cfg(test)]
mod test {
    use super::AST;
    use crate::parsing::{Parser, ParserError};

    fn diff_same_as_tostring(str1: &str, str2: &str) -> Result<(), ParserError> {
        let ast1 = Parser::from_string(str1.to_string()).parse_module(true)?.ast;
        let ast2 = Parser::from_string(str2.to_string()).parse_module(true)?.ast;

        let ast1_main = ast1.get_assign_exp(ast1.get_main(ast1.root).unwrap());
        let ast2_main = ast1.get_assign_exp(ast1.get_main(ast1.root).unwrap());

        let diff = AST::diff(&ast1, &ast2, ast1_main, ast2_main);
        assert_eq!(diff.str_1(), ast1.to_string_sugar(ast1_main, false));
        assert_eq!(diff.str_2(), ast2.to_string_sugar(ast2_main, false));
        Ok(())
    }

    #[test]
    fn diff_test1() -> Result<(), ParserError> {     
        diff_same_as_tostring(r#"
            f :: Int -> Int
            f n = if (n % 2) == 0 then n / 2 else (3 * n) + 1

            // Get collatz sequence
            collatz :: Int -> List Int
            collatz n = (\x. if n <= 1 then Nil else Cons x (collatz x)) $ f n

            main :: List Int
            main = collatz 12
        "#, r#"
            f :: Int -> Int
            f n = if (n % 2) == 0 then n / 2 else (3 * n) + 1

            // Get collatz sequence
            collatz :: Int -> List Int
            collatz n = (\x. if n <= 1 then Nil else Cons x (collatz x)) $ f n

            main :: List Int
            main = \x. if 12 <= 1 then Nil else Cons x (collatz x) $ f 12
        "#)
    }
}
