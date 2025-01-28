use crate::{find_redexes::RCPair, types::TypeError, Primitive, Type};
use std::collections::HashSet;
use std::{collections::HashMap, fmt::Debug, vec};

use super::token::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,
    Pair,
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
    pub wait_for_args: bool,
    pub fancy_assign_abst_syntax: bool,
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{:?} ", self.t);
        if let Some(tk) = &self.info {
            s.push_str(&format!("{:?} ", tk.value));
        }
        // s.push_str(&format!("{}:{} ", self.line, self.col));
        // if let Some(t) = &self.type_assignment {
        //     s.push_str(&format!("Type: {:?} ", t));
        // }
        // s.push_str(&format!("Children: {:?}", self.children));
        write!(f, "{}", s)
    }
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Primitive(Primitive::Int64),
                    TokenType::FloatLit => Type::Primitive(Primitive::Float64),
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
    #[inline(always)]
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
        }
    }

    fn new_pair(a: usize, b: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Pair,
            info: None,
            children: vec![a, b],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
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
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
        }
    }

    fn wait_for_args(&mut self) {
        self.wait_for_args = true;
    }
}

impl AST {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            root: 0,
        }
    }

    pub fn wait_for_args(&mut self, node: usize) {
        self.vec[node].wait_for_args();
    }

    pub fn fancy_assign_abst_syntax(&mut self, node: usize) {
        self.vec[node].fancy_assign_abst_syntax = true;
    }

    pub fn set_type(&mut self, node: usize, t: Type) {
        self.vec[node].type_assignment = Some(t);
    }

    pub fn add(&mut self, n: ASTNode) -> usize {
        self.vec.push(n);
        self.vec.len() - 1
    }

    pub fn expr_eq(&self, n1: usize, n2: usize) -> bool {
        match (&self.get(n1).t, &self.get(n2).t) {
            (ASTNodeType::Identifier, ASTNodeType::Identifier)
            | (ASTNodeType::Literal, ASTNodeType::Literal) => {
                self.get(n1).get_value() == self.get(n2).get_value()
            }
            (ASTNodeType::Application, ASTNodeType::Application) => {
                let f1 = self.get_func(n1);
                let f2 = self.get_func(n2);
                let x1 = self.get_arg(n1);
                let x2 = self.get_arg(n2);

                self.expr_eq(f1, f2) && self.expr_eq(x1, x2)
            }
            (ASTNodeType::Abstraction, ASTNodeType::Abstraction) => {
                let v1 = self.get_abstr_var(n1);
                let v2 = self.get_abstr_var(n2);
                let x1 = self.get_abstr_expr(n1);
                let x2 = self.get_abstr_expr(n2);

                self.expr_eq(v1, v2) && self.expr_eq(x1, x2)
            }
            (ASTNodeType::Pair, ASTNodeType::Pair) => {
                let x1 = self.get_first(n1);
                let y1 = self.get_second(n1);
                let x2 = self.get_first(n2);
                let y2 = self.get_second(n2);

                self.expr_eq(x1, x2) && self.expr_eq(y1, y2)
            }
            _ => false,
        }
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

    pub fn do_rc_subst(&mut self, rc: &RCPair) {
        let other = &rc.1;
        let old = rc.0;
        let new = self.append(other, other.root);

        #[cfg(debug_assertions)]
        let _old_str = self.to_string_sugar(old, false);
        #[cfg(debug_assertions)]
        let _new_str = self.to_string_sugar(new, false);
        self.replace_references_to_node(old, new);
    }

    pub fn filter_identical_rcs(&self, rcs: &Vec<RCPair>) -> Vec<RCPair> {
        let mut stringset = HashSet::new();
        let mut new_rcs = vec![];
        for rc in rcs {
            let str = self.to_string_sugar(rc.0, false);
            if !stringset.contains(&str) {
                new_rcs.push(rc.clone());
            }
            stringset.insert(str);
        }
        new_rcs
    }

    pub fn do_rc_subst_and_identical_rcs(&mut self, rc0: &RCPair, rcs: &Vec<RCPair>) {
        self.do_rc_subst_and_identical_rcs_borrowed(rc0, &rcs.into_iter().map(|rc| rc).collect());
    }

    pub fn do_rc_subst_and_identical_rcs_borrowed(&mut self, rc0: &RCPair, rcs: &Vec<&RCPair>) {
        #[cfg(debug_assertions)]
        let _rc0_0_str = self.to_string_sugar(rc0.0, false);
        #[cfg(debug_assertions)]
        let _rc1_0_str = rc0.1.to_string_sugar(rc0.1.root, false);

        for rc in rcs {
            #[cfg(debug_assertions)]
            let _this_rc = self.to_string_sugar(rc.0, false);
            #[cfg(debug_assertions)]
            let _this_rc_1 = rc.1.to_string_sugar(rc.1.root, false);
            if self.expr_eq(rc0.0, rc.0) {
                self.do_rc_subst(rc);
            }
        }
    }

    pub fn do_rc_subst_and_identical_substs_borrowed(&mut self, rc0: &RCPair) {
        #[cfg(debug_assertions)]
        let _rc0_0_str = self.to_string_sugar(rc0.0, false);
        #[cfg(debug_assertions)]
        let _rc1_0_str = rc0.1.to_string_sugar(rc0.1.root, false);


        // Its important that the ast contains no orphans before this function is called
        for node in 0..self.vec.len() {
            if self.expr_eq(rc0.0, node) {
                self.do_rc_subst(&(node, rc0.1.clone()));
            }
        }
    }

    pub fn rc_to_str(&self, rc: &RCPair) -> String {
        self.to_string_sugar(rc.0, false) + " -> " + &rc.1.to_string_sugar(rc.1.root, false)
    }

    pub fn print_vec_string(&self) -> String {
        let mut s = String::new();
        for n in &self.vec {
            s.push_str(&format!("{:?}\n", n));
        }
        s
    }

    fn get_laziest_rc_recurse(
        &self,
        expr: usize,
        rc_map: &HashMap<usize, &RCPair>,
    ) -> Option<RCPair> {
        if rc_map.contains_key(&expr) {
            return Some(rc_map[&expr].clone());
        }

        #[cfg(debug_assertions)]
        let _expr_str = self.to_string_sugar(expr, false);
        #[cfg(debug_assertions)]
        let _rcs_strs = {
            let mut _rcs_strs = vec![];
            for rc in rc_map.values() {
                _rcs_strs.push(self.to_string_sugar(rc.0, false));
            }
            _rcs_strs
        };

        match self.get(expr).t {
            ASTNodeType::Application | ASTNodeType::Pair => {
                let f = self.get(expr).children[0];
                let x = self.get(expr).children[1];

                if let Some(rc) = self.get_laziest_rc_recurse(f, &rc_map) {
                    return Some(rc);
                }

                self.get_laziest_rc_recurse(x, &rc_map)
            }
            _ => None,
        }
    }

    pub fn get_laziest_rc(&self, expr: usize, rcs: &Vec<RCPair>) -> Option<RCPair> {
        self.get_laziest_rc_borrowed(expr, &rcs.into_iter().map(|rc| rc).collect())
    }

    pub fn get_laziest_rc_borrowed(&self, expr: usize, rcs: &Vec<&RCPair>) -> Option<RCPair> {
        // Convert to map for O(1) lookup of whether a node is an RC
        let mut rc_map: HashMap<usize, &RCPair> = HashMap::new();
        for rc in rcs {
            rc_map.insert(rc.0, &rc);
        }

        if rcs.is_empty() {
            None
        } else {
            self.get_laziest_rc_recurse(expr, &rc_map)
        }
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
                let exp = self.append(other, other.get_abstr_expr(node));
                self.add_abstraction(var, exp, n.line, n.col)
            }
            ASTNodeType::Module => {
                let mut assigns = vec![];
                for a in n.children.clone() {
                    assigns.push(self.append(other, a));
                }
                self.add_module(assigns, n.line, n.col)
            }
            ASTNodeType::Pair => {
                let a = self.append(other, n.children[0]);
                let b = self.append(other, n.children[1]);
                self.add_pair(a, b, n.line, n.col)
            }
        }
    }

    pub fn append_root(&mut self, other: &AST) -> usize {
        self.append(other, other.root)
    }

    pub fn add_id(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_id(tk, line, col))
    }

    pub fn add_typed_id(
        &mut self,
        tk: Token,
        line: usize,
        col: usize,
        assigned_type: Type,
    ) -> usize {
        let node = self.add(ASTNode::new_id(tk, line, col));
        self.vec[node].type_assignment = Some(assigned_type);
        node
    }

    pub fn add_lit(&mut self, tk: Token, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_lit(tk, line, col))
    }

    pub fn add_app(&mut self, f: usize, x: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_app(f, x, line, col))
    }

    pub fn add_pair(&mut self, a: usize, b: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_pair(a, b, line, col))
    }

    pub fn add_abstraction(&mut self, id: usize, exp: usize, line: usize, col: usize) -> usize {
        self.add(ASTNode::new_abstraction(id, exp, line, col))
    }

    pub fn set_assignment_type(&mut self, assignment: usize, type_: Type) {
        self.vec[assignment].type_assignment = Some(type_);
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
        assert_eq!(self.vec[module].t, ASTNodeType::Module);
        self.vec[module].children.push(assign);
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> &ASTNode {
        &self.vec[i]
    }

    pub fn get_first(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[0]
    }

    pub fn get_second(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[1]
    }

    pub fn get_abstr_var(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
        self.vec[abst].children[0]
    }

    pub fn get_abstr_expr(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
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

                self.get_all_instances_of_var_in_exp(self.get_abstr_expr(exp), var)
            }
            ASTNodeType::Pair => {
                let mut left = self.get_all_instances_of_var_in_exp(self.get_first(exp), &var);
                let right = self.get_all_instances_of_var_in_exp(self.get_second(exp), &var);
                left.extend(right);
                left
            }
            _ => unreachable!("Cannot find var instances in non exp"),
        }
    }

    pub fn get_abst_var_usages(&self, abst: usize) -> Vec<usize> {
        let var_name = self.get(self.get_abstr_var(abst)).get_value();
        self.get_all_instances_of_var_in_exp(self.get_abstr_expr(abst), &var_name)
    }

    pub fn get_n_abstr_vars(&self, abstr: usize, n: usize) -> Vec<usize> {
        if n <= 0 {
            vec![]
        } else {
            let var = self.get_abstr_var(abstr);
            let mut expr = self.get_n_abstr_vars(abstr, n - 1);
            expr.insert(0, var);
            expr
        }
    }

    pub fn do_multiple_abst_substs(&self, abst: usize, substs: Vec<usize>) -> Self {
        assert!(substs.len() > 0);

        let mut abst_ast = self.do_abst_subst(abst, *substs.last().unwrap());
        let substs = &substs[..substs.len() - 1];
        for subst in substs {
            let subst = abst_ast.append(self, *subst);
            abst_ast = abst_ast.do_abst_subst(abst_ast.root, subst);
        }

        abst_ast
    }

    fn replace_var_usages(&mut self, var: usize, subst: usize) {
        #[cfg(debug_assertions)]
        let _var_str = self.to_string_sugar(var, false);
        #[cfg(debug_assertions)]
        let _subst_str = self.to_string_sugar(subst, false);

        let var_n = self.get(var);
        match var_n.t {
            ASTNodeType::Identifier => {
                let var_name = self.get(var).get_value();
                let usages =
                    self.get_all_instances_of_var_in_exp(self.get_abstr_expr(self.root), &var_name);
                for usage in usages {
                    self.replace(usage, subst);
                }
            }
            ASTNodeType::Pair => {
                let subst_first = self.get_first(subst);
                let subst_second = self.get_second(subst);
                self.replace_var_usages(self.get_first(var), subst_first);
                self.replace_var_usages(self.get_second(var), subst_second);
            }
            _ => unreachable!("WTF HOW DID THIS HAPPEN"),
        }
    }

    pub fn do_abst_subst(&self, abstr: usize, subst: usize) -> Self {
        assert_eq!(self.get(abstr).t, ASTNodeType::Abstraction);
        let mut cloned_abst_expr = self.clone_node(abstr);
        let new_abstr_var = cloned_abst_expr.get_abstr_var(cloned_abst_expr.root);
        let subst_id = cloned_abst_expr.append(&self, subst);

        cloned_abst_expr.replace_var_usages(new_abstr_var, subst_id);
        let _abst_str = cloned_abst_expr.to_string_sugar(cloned_abst_expr.root, false);
        cloned_abst_expr.root = cloned_abst_expr.get_abstr_expr(cloned_abst_expr.root);
        let _abst_str = cloned_abst_expr.to_string_sugar(cloned_abst_expr.root, false);
        cloned_abst_expr
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

    pub fn get_main(&self, module: usize) -> Option<usize> {
        self.get_assign_to(module, "main".to_string())
    }

    // Get assignment within module
    pub fn get_assign_to(&self, module: usize, name: String) -> Option<usize> {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);

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
        assert_eq!(self.vec[module].t, ASTNodeType::Module);
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
            ASTNodeType::Pair => {
                let a = self.to_string_indent(self.get_first(node), indent + 2);
                let b = self.to_string_indent(self.get_second(node), indent + 2);
                format!("{}Pair: {}\n{}", ind, a, b)
            }
        }
    }

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

                // If the func is an abstraction, wrap it in parens
                let func_str = match self.get(func).t {
                    ASTNodeType::Abstraction => format!("({})", func_str),
                    _ => func_str,
                };

                let arg_str = self.to_string_sugar(arg, show_assigned_types);
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
                res.push_str(" . ");
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
        write!(f, "{}", self.to_string_sugar(self.root, false))
    }
}
