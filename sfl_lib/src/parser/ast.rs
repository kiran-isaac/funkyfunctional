use std::{collections::HashMap, fmt::Debug, vec};

use crate::{types::TypeError, Primitive, Type};

use super::token::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,

    Application,
    Assignment,

    Abstraction,

    Module,
}

#[derive(Clone)]
pub struct AST {
    vec: Vec<ASTNode>,
    pub root: usize,
}

#[derive(Clone)]
pub struct ASTNode {
    pub t: ASTNodeType,
    info: Option<Token>,
    children: Vec<usize>,
    pub line: usize,
    pub col: usize,
    pub type_assignment: Option<Type>,
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Primitive(Primitive::Int64),
                    TokenType::FloatLit => Type::Primitive(Primitive::Float64),
                    TokenType::CharLit => Type::Primitive(Primitive::Char),
                    TokenType::BoolLit => Type::Primitive(Primitive::Bool),
                    TokenType::StringLit => unimplemented!("String literal type"),
                    _ => unreachable!("Literal node with bad token"),
                },
                None => unreachable!("Literal node with no token"),
            },
            _ => unreachable!("get_lit_type called on non-literal node"),
        }
    }

    /// Get the string value of the identifier or literal
    pub fn get_value(&self) -> String {
        assert!(self.t == ASTNodeType::Identifier || self.t == ASTNodeType::Literal);
        match &self.info {
            Some(tk) => tk.value.clone(),
            None => unreachable!(),
        }
    }

    fn new_lit(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
        }
    }

    fn new_id(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
        }
    }

    fn new_app(f: usize, x: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, x],
            line,
            col,
            type_assignment: None,
        }
    }

    fn new_abstraction(id: usize, exp: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Abstraction,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: None,
        }
    }

    fn new_assignment(id: usize, exp: usize, line: usize, col: usize, t: Option<Type>) -> Self {
        ASTNode {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: t,
        }
    }

    fn new_module(assigns: Vec<usize>, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Module,
            info: None,
            children: assigns,
            line,
            col,
            type_assignment: None,
        }
    }
}

impl AST {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            root: 0,
        }
    }

    pub fn add(&mut self, n: ASTNode) -> usize {
        self.vec.push(n);
        self.vec.len() - 1
    }

    pub fn single_node(n: ASTNode) -> Self {
        let mut ast = Self::new();
        let id = ast.add(n);
        ast.root = id;

        ast
    }

    pub fn clone_node(&self, n: usize) -> AST {
        let node = self.get(n);
        let mut ast = AST::single_node(node.clone());
        for i in 0..node.children.len() {
            let index = ast.append_root(&self.clone_node(node.children[i]));
            ast.vec[ast.root].children[i] = index;
        }

        ast
    }

    pub fn do_rc_subst(&mut self, rc : &(usize, AST)) {
        let other = &rc.1;
        let old = rc.0;
        let new = self.append(other, other.root);
        self.replace_references_to_node(old, new);
    }

    pub fn replace(&mut self, old: usize, new: usize) {
        // Replace references to the old node with the new node
        self.replace_references_to_node(old, new);
    }

    fn replace_references_to_node(&mut self, old: usize, new: usize) {
        if self.root == old {
            self.root = new;
        }
        for n in &mut self.vec {
            for c in &mut n.children {
                if *c == old {
                    *c = new;
                }
            }
        }
    }

    // Add a node from another ast to this ast with its children
    pub fn append(&mut self, other: &AST, node: usize) -> usize {
        let n = other.get(node);
        match n.t {
            ASTNodeType::Identifier => self.add_id(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Literal => self.add_lit(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Application => {
                let f = self.append(other, other.get_func(node));
                let x = self.append(other, other.get_arg(node));
                self.add_app(f, x, n.line, n.col)
            }
            ASTNodeType::Assignment => {
                let id = self.append(other, n.children[0]);
                let exp = self.append(other, other.get_assign_exp(node));
                self.add_assignment(id, exp, n.line, n.col, n.type_assignment.clone())
            }
            ASTNodeType::Abstraction => {
                let var = self.append(other, n.children[0]);
                let exp = self.append(other, other.get_abstr_exp(node));
                self.add_abstraction(var, exp, n.line, n.col)
            }
            ASTNodeType::Module => {
                let mut assigns = vec![];
                for a in n.children.clone() {
                    assigns.push(self.append(other, a));
                }
                self.add_module(assigns, n.line, n.col)
            }
        }
    }

    pub fn append_root(&mut self, other: &AST) -> usize {
        self.append(other, other.root)
    }

    pub fn add_id(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_id(tk, line, col))
    }

    pub fn add_lit(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_lit(tk, line, col))
    }

    pub fn add_app(&mut self, f: usize, x: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_app(f, x, line, col))
    }

    pub fn add_abstraction(&mut self, id: usize, exp: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_abstraction(id, exp, line, col))
    }

    pub fn add_assignment(
        &mut self,
        id: usize,
        exp: usize,
        line: usize,
        col: usize,
        t: Option<Type>,
    ) -> usize {
        self.add(ASTNode::new_assignment(id, exp, line, col, t))
    }

    pub fn add_module(&mut self, assigns: Vec<usize>, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_module(assigns, line, col))
    }

    pub fn add_to_module(&mut self, module: usize, assign: usize) {
        assert!(self.vec[module].t == ASTNodeType::Module);
        self.vec[module].children.push(assign);
    }

    pub fn get(&self, i: usize) -> &ASTNode {
        &self.vec[i]
    }

    pub fn get_abstr_var(&self, abst: usize) -> usize {
        assert!(self.vec[abst].t == ASTNodeType::Abstraction);
        self.vec[abst].children[0]
    }

    pub fn get_abstr_exp(&self, abst: usize) -> usize {
        assert!(self.vec[abst].t == ASTNodeType::Abstraction);
        self.vec[abst].children[1]
    }

    pub fn get_all_instances_of_var_in_exp(&self, exp: usize, var: &String) -> Vec<usize> {
        match self.get(exp).t {
            ASTNodeType::Literal => {
                vec![]
            }
            ASTNodeType::Identifier => {
                if var == &self.get(exp).get_value() {
                    vec![exp]
                } else {
                    vec![]
                }
            }
            ASTNodeType::Application => {
                let mut left = self.get_all_instances_of_var_in_exp(self.get_func(exp), &var);
                let right = self.get_all_instances_of_var_in_exp(self.get_arg(exp), &var);
                left.extend(right);
                left
            }
            ASTNodeType::Abstraction => {
                let abst_var = self.get_abstr_var(exp);
                assert_ne!(&self.get(abst_var).get_value(), var);

                self.get_all_instances_of_var_in_exp(self.get_abstr_exp(exp), var)
            }
            _ => unreachable!("Cannot find var instances in non exp"),
        }
    }

    pub fn get_abst_var_usages(&self, abst: usize) -> Vec<usize> {
        let var_name = self.get(self.get_abstr_var(abst)).get_value();
        self.get_all_instances_of_var_in_exp(self.get_abstr_exp(abst), &var_name)
    }

    pub fn get_func(&self, app: usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[0]
    }

    pub fn get_arg(&self, app: usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[1]
    }

    pub fn get_assign_exp(&self, assign: usize) -> usize {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
        self.vec[assign].children[1]
    }

    pub fn get_assignee(&self, assign: usize) -> String {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
        self.get(self.vec[assign].children[0]).get_value().clone()
    }

    pub fn get_assignee_names(&self, module: usize) -> Vec<String> {
        let mut names = Vec::new();
        let assigns = &self.vec[module].children;
        names.reserve_exact(assigns.len());
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            names.push(id.get_value());
        }

        names
    }

    pub fn get_main(&self, module: usize) -> usize {
        self.get_assign_to(module, "main".to_string()).unwrap()
    }

    // Get assignment within module
    pub fn get_assign_to(&self, module: usize, name: String) -> Option<usize> {
        assert!(self.vec[module].t == ASTNodeType::Module);

        let assigns = &self.vec[module].children;
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            if id.get_value() == name {
                return Some(*a);
            }
        }

        None
    }

    pub fn get_assigns_map(&self, module: usize) -> HashMap<String, usize> {
        assert!(self.vec[module].t == ASTNodeType::Module);
        let mut assigns = HashMap::new();

        for a in &self.vec[module].children {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            assigns.insert(id.get_value(), *a);
        }

        assigns
    }

    fn to_string_indent(&self, node: usize, indent: usize) -> String {
        let n = self.get(node);
        let ind = " ".repeat(indent);
        match n.t {
            ASTNodeType::Identifier => {
                format!("{}Identifier: {}", ind, n.get_value())
            }
            ASTNodeType::Literal => {
                format!("{}Literal: {}", ind, n.get_value())
            }
            ASTNodeType::Application => {
                let left = self.to_string_indent(self.get_func(node), indent + 2);
                let right = self.to_string_indent(self.get_arg(node), indent + 2);
                format!("{}Application\n{}\n{}", ind, left, right)
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let exp = self.to_string_indent(self.get_assign_exp(node), indent + 2);
                format!("{}Assignment: {}\n{}", ind, id.get_value(), exp)
            }
            ASTNodeType::Module => {
                let mut s = format!("{}Module\n", ind);
                for c in &n.children {
                    s.push_str(&self.to_string_indent(*c, indent + 2));
                }
                s
            }
            ASTNodeType::Abstraction => {
                let id = self.get(n.children[0]);
                let exp = self.to_string_indent(n.children[1], indent + 2);
                format!("{}Abstraction: {}\n{}", ind, id.get_value(), exp)
            }
        }
    }

    pub fn to_string(&self, node: usize) -> String {
        let n = self.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                format!("{}", n.get_value())
            }
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
                                    self.to_string(args[2]),
                                    self.to_string(args[1]),
                                    self.to_string(args[0])
                                );
                            }
                        }
                        _ => {}
                    }
                }

                let func = self.get_func(node);
                let arg = self.get_arg(node);

                let func_str = self.to_string(func);

                // If the func is an abstraction, wrap it in parens
                let func_str = match self.get(func).t {
                    ASTNodeType::Abstraction => format!("({})", func_str),
                    _ => func_str,
                };

                let arg_str = self.to_string(arg);
                // If the argument is an application, wrap it in parens
                let arg_str = match self.get(arg).t {
                    ASTNodeType::Application | ASTNodeType::Abstraction => format!("({})", arg_str),
                    _ => arg_str,
                };

                format!("{} {}", func_str, arg_str)
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let exp = self.to_string(self.get_assign_exp(node));
                format!("{} = {}", id.get_value(), exp)
            }
            ASTNodeType::Module => {
                let mut s = String::new();
                for c in &n.children {
                    s.push_str(&self.to_string(*c));
                    s.push_str("\n");
                }

                s.trim().to_string()
            }
            ASTNodeType::Abstraction => {
                let id = self.get(n.children[0]);
                let exp = self.to_string(n.children[1]);
                format!("\\{} . {}", id.get_value(), exp)
            }
        }
    }

    pub fn display_string(&self, node: usize) -> String {
        self.to_string_indent(node, 0)
    }

    fn type_error(&self, e: String, node: usize) -> TypeError {
        let n = self.get(node);
        TypeError {
            e,
            line: n.line,
            col: n.col,
        }
    }

    pub fn get_type(&self, node: usize) -> Result<Type, TypeError> {
        match self.get(node).t {
            ASTNodeType::Literal => Ok(self.get(node).get_lit_type()),
            _ => Err(self.type_error("Smoingus".to_string(), node)),
        }
    }
}

impl Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(self.root))
    }
}
