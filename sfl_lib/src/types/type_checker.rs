use std::iter::zip;

use crate::{functions::KnownTypeLabelTable, parser::TypeMap, ASTNodeType, AST};

use super::{Type, TypeError};

#[derive(Clone, PartialEq, Eq)]
enum ContextItem {
    TypeVariable(String),
    TypeAssignment(String, Result<Type, TypeError>),
    Existential(usize, Option<Type>),
    Marker(usize),
}

impl std::fmt::Debug for ContextItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextItem::TypeVariable(v) => write!(f, "{}", v),
            ContextItem::Existential(v, ass) => match ass {
                Some(t) => write!(f, "{}:{}", Type::Existential(*v).to_string(), t.to_string()),
                None => write!(f, "{}", Type::Existential(*v).to_string()),
            },
            ContextItem::Marker(v) => write!(f, "|{}|", Type::Existential(*v).to_string()),
            ContextItem::TypeAssignment(name, tr) => match tr {
                Ok(t) => write!(f, "{}:{}", name, t.to_string()),
                Err(_) => write!(f, "{}:ERROR", name),
            },
        }
    }
}

#[derive(Clone)]
struct Context {
    vec: Vec<ContextItem>,
    next_exid: usize,
    next_placeholder_assignvar_i: usize,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inbuilt_map = KnownTypeLabelTable::new().get_type_map();
        write!(f, "[")?;
        for item in &self.vec {
            match item {
                ContextItem::TypeAssignment(name, _) => {
                    if !inbuilt_map.contains_key(name) {
                        write!(f, "{:?}, ", item)?;
                    }
                }
                _ => write!(f, "{:?}, ", item)?,
            }
        }
        write!(f, "]")
    }
}

impl Context {
    fn from_labels(labels: &KnownTypeLabelTable) -> Self {
        let mut vec = vec![];

        for (k, v) in labels.get_type_map() {
            vec.push(ContextItem::TypeAssignment(k.clone(), Ok(v.clone())));
        }

        Self { vec, next_exid: 0, next_placeholder_assignvar_i: 0 }
    }

    fn assigns_only(&self) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(_, _) => {
                    new_v.push(i.clone());
                }
                _ => {}
            }
        }

        Self { vec: new_v, next_exid: self.next_exid, next_placeholder_assignvar_i: self.next_placeholder_assignvar_i }
    }

    fn append(&self, item: ContextItem) -> Self {
        let mut new = self.clone();

        match &item {
            ContextItem::TypeAssignment(name, _) => {
                if name.starts_with("_") {
                    new.next_placeholder_assignvar_i += 1;
                }
            }
            ContextItem::Existential(e, _) => {
                new.next_exid = std::cmp::max(*e, new.next_exid);
            }
            _ => {}
        }

        new.vec.push(item);
        new
    }

    fn remove_assignment(&self, name: &String) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            if let ContextItem::TypeAssignment(name2, _) = i {
                if name == name2 {
                    continue;
                }
            }

            new_v.push(i.clone());
        }

        Self { vec: new_v, next_exid: self.next_exid, next_placeholder_assignvar_i: self.next_placeholder_assignvar_i  }
    }

    fn get_before_item(&self, item: ContextItem) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            if i == &item {
                break;
            }

            new_v.push(i.clone());
        }

        Self { vec: new_v, next_exid: self.next_exid, next_placeholder_assignvar_i: self.next_placeholder_assignvar_i  }
    }
    fn get_before_assignment(&self, str: String) -> Self {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &self);
        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(v, _) => {
                    if v == &str {
                        break;
                    }
                }
                _ => {}
            }

            new_v.push(i.clone());
        }

        let new_s = Self { vec: new_v, next_exid: self.next_exid, next_placeholder_assignvar_i: self.next_placeholder_assignvar_i };

        #[cfg(debug_assertions)]
        let _new_s_str = format!("{:?}", &new_s);
        new_s
    }

    fn get_type_assignment(&self, var: &str) -> Option<Result<Type, TypeError>> {
        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(v, t) => {
                    if v == var {
                        return Some(t.clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn add_before_existential(&self, existential: usize, item: ContextItem) -> Self {
        let mut new_v = vec![];
        let mut next_placeholder_assignvar_i = self.next_placeholder_assignvar_i;
        let mut next_exid = self.next_exid;

        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(name, _) => {
                    if name.starts_with("_") {
                        next_placeholder_assignvar_i += 1;
                    }
                }
                ContextItem::Existential(e, _) => {
                    if *e == existential {
                        new_v.push(item.clone());
                    }
                    next_exid = std::cmp::max(*e, next_exid);
                }
                _ => {}
            }
            new_v.push(i.clone());
        }

        Self { vec: new_v, next_exid, next_placeholder_assignvar_i  }
    }

    fn set_existential_definition(&self, existential_being_set: usize, t: Type) -> Self {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &self);
        #[cfg(debug_assertions)]
        let _t_str = format!("{:?}", &t);

        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                // If this is another existential that references the one being substituted then
                // Substitute this one too
                ContextItem::Existential(e, Some(Type::Existential(e2))) => {
                    if *e2 == existential_being_set {
                        new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                        continue;
                    }
                }
                // If this is an existential with a product that refers to the one being set, set the
                // types within the product too
                // ContextItem::Existential(e, Some(Type::Product(t1, t2))) => {
                //     let mut t1 = t1.as_ref().clone();
                //     if let Type::Existential(e1) = t1 {
                //         if e1 == existential_being_set {
                //             t1 = t.clone();
                //         }
                //     }
                //
                //     let mut t2 = t2.as_ref().clone();
                //     if let Type::Existential(e2) = t2 {
                //         if e2 == existential_being_set {
                //             t2 = t.clone();
                //         }
                //     }
                //     new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                //     continue;
                // }
                ContextItem::Existential(e, _) => {
                    if *e == existential_being_set {
                        new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                        continue;
                    }
                }
                _ => {}
            }
            new_v.push(i.clone());
        }

        #[cfg(debug_assertions)]
        let _new_v_str = format!("{:?}", new_v);

        Self { vec: new_v, next_exid: self.next_exid, next_placeholder_assignvar_i: self.next_placeholder_assignvar_i  }
    }

    fn get_next_existential_identifier(&self) -> usize {
        self.next_exid + 1
    }

    fn get_next_placeholder_assignvar(&self) -> String {
        "_".to_string() + &self.next_placeholder_assignvar_i.to_string()
    }

    // Stupid name
    fn get_existential(&self, ex: usize) -> Option<Option<Type>> {
        for i in &self.vec {
            match i {
                ContextItem::Existential(ex2, o) => {
                    if ex == *ex2 {
                        return Some(o.clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn substitute(&self, t: &Type) -> Type {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", self);
        #[cfg(debug_assertions)]
        let _t_str = format!("{}", t.to_string());

        match t {
            Type::Existential(ex) => match self.get_existential(*ex) {
                Some(o) => match o {
                    Some(t2) => self.substitute(&t2.clone()),
                    None => t.clone(),
                },
                None => {
                    t.clone()
                }
            },
            Type::Function(from, to) => Type::Function(
                Box::new(self.substitute(from.as_ref())),
                Box::new(self.substitute(to.as_ref())),
            ),
            Type::Product(t1, t2) => {
                Type::Product(Box::new(self.substitute(t1)), Box::new(self.substitute(t2)))
            }
            Type::Forall(var, t) => {
                Type::Forall(var.clone(), Box::new(self.substitute(t.as_ref())))
            }
            Type::Union(name, types) => {
                let mut new_types = vec![];
                for t in types {
                    new_types.push(self.substitute(t));
                }
                Type::Union(name.clone(), new_types)
            }
            _ => t.clone(),
        }
    }
}

fn type_error(msg: String, ast: &AST, expr: usize) -> TypeError {
    let n = ast.get(expr);
    TypeError {
        e: msg,
        col: n.col,
        line: n.line,
    }
}

fn subtype(c: Context, a: &Type, b: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _a_str = a.to_string();
    #[cfg(debug_assertions)]
    let _b_str = b.to_string();

    match (a, b) {
        (Type::Alias(_, a_type), _) => {
            subtype(c, a_type, b, type_map)
        }
        (_, Type::Alias(_, a_type)) => {
            subtype(c, a, a_type, type_map)
        }

        // <:InstantiateL
        (Type::Existential(ex), _) => {
            match b {
                Type::Existential(ex2) => {
                    if ex == ex2 {
                        return Ok(c)
                    }
                }
                _ => {}
            }

            if b.contains_existential(*ex) {
                let a = c.substitute(a);
                let b = c.substitute(b);
                return Err(format!("Cannot instantiate existential variable {} to the type {}: the type contains the existential type variable in question!", a, b));
            }

            instantiate_l(c, *ex, b, type_map)
        }

        // <:InstantiateR
        (_, Type::Existential(ex)) => {
            if a.contains_existential(*ex) {
                let a = c.substitute(a);
                let b = c.substitute(b);
                return Err(format!("Cannot instantiate existential variable {} to the type {}: the type contains the existential type variable in question!", b, a));
            }

            instantiate_r(c, *ex, a, type_map)
        }

        // <:Var
        (Type::TypeVariable(a), Type::TypeVariable(b)) => {
            if a == b {
                Ok(c)
            } else {
                Err(format!("{} is not a subtype of {}", a, b))
            }
        }

        (Type::Primitive(a), Type::Primitive(b)) => {
            if a == b {
                Ok(c)
            } else {
                Err(format!("{} is not a subtype of {}", a, b))
            }
        }

        // <:Unit
        (Type::Unit, Type::Unit) => Ok(c),

        // <:ForallL
        (Type::Forall(var, t), _) => {
            let exst = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Marker(exst))
                .append(ContextItem::Existential(exst, None));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let new_body = t.substitute_type_variable(var, &Type::Existential(exst))?;
            let pred = subtype(c, &new_body, b, type_map)?;
            Ok(pred.get_before_item(ContextItem::Marker(exst)))
        }

        // <:ForallR
        (_, Type::Forall(var, t)) => {
            let c = c.append(ContextItem::TypeVariable(var.clone()));
            let pred = subtype(c, a, t.as_ref(), type_map)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        // <:->
        (Type::Function(a1, a2), Type::Function(b1, b2)) => {
            let pred1 = subtype(c, b1.as_ref(), a1, type_map)?;
            let a2 = &pred1.substitute(a2);
            let b2 = &pred1.substitute(b2);
            let pred2 = subtype(pred1, a2, b2, type_map)?;
            Ok(pred2)
        }

        (Type::Product(pt1_1, pt1_2), a) | (a, Type::Product(pt1_1, pt1_2)) => {
            let (ut2_1, ut2_2) = match a {
                Type::Product(a, b) => (a, b),
                _ => {
                    return Err(format!(
                        "Type {} is not a subtype of product {}",
                        a,
                        Type::Product(pt1_1.clone(), pt1_2.clone())
                    ))
                }
            };
            let ut_1_st = subtype(c, pt1_1, ut2_1, type_map)?;
            subtype(ut_1_st, pt1_2, ut2_2, type_map)
        }

        (Type::Union(name1, uargs1), Type::Union(name2, uargs2)) => {
            if uargs1.len() != uargs2.len() || name1 != name2 {
                return Err(format!(
                    "Type {} is not a subtype of union {}",
                    a,
                    Type::Union(name1.clone(), uargs1.clone())
                ))
            }
            let mut c = c;
            for (t1, t2) in zip(uargs1, uargs2) {
                c = subtype(c, t1, t2, type_map)?;
            }
            Ok(c)
        },

        _ => Err("Subtype failiure".to_string()),
    }
}

fn instantiate_l(c: Context, exst: usize, b: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _exst_str = format!("{}", Type::Existential(exst).to_string());
    #[cfg(debug_assertions)]
    let _b_str = format!("{}", &b.to_string());
    match b {
        // InstLReach
        Type::Existential(exst2) => {
            Ok(c.set_existential_definition(*exst2, Type::Existential(exst)))
        }

        // InstLArr
        Type::Function(from, to) => {
            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            let c = c
                .add_before_existential(exst, a1)
                .add_before_existential(exst, a2);

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let pred1_c = instantiate_r(c, exst, from.as_ref(), type_map)?;
            let to_subst = pred1_c.substitute(to);

            #[cfg(debug_assertions)]
            let _to_subst_str = format!("{:?}", &to_subst);

            let pred2 = instantiate_l(pred1_c, a2n, &to_subst, type_map)?;
            Ok(pred2)
        }

        // InstLAllR
        Type::Forall(var, t) => {
            let new_c = c.append(ContextItem::TypeVariable(var.clone()));
            let pred = instantiate_l(new_c, exst, t.as_ref(), type_map)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        _ => {
            if !b.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            if let Some(existential) = c.get_existential(exst) {
                if let Some(existential_type_assignment) = existential {
                    subtype(c.clone(), &existential_type_assignment, &b.clone(), type_map)?;
                }
            }

            // InstLSolve
            Ok(c.set_existential_definition(exst, b.clone()))
        }
    }
}

fn instantiate_r(c: Context, exst: usize, a: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _exst_str = format!("{}", Type::Existential(exst).to_string());
    #[cfg(debug_assertions)]
    let _a_str = format!("{}", &a.to_string());
    match a {
        // InstRReach
        Type::Existential(exst2) => {
            Ok(c.set_existential_definition(*exst2, Type::Existential(exst)))
        }

        // InstRArr
        Type::Function(from, to) => {
            #[cfg(debug_assertions)]
            let _from_str = format!("{}", &from.to_string());
            #[cfg(debug_assertions)]
            let _to_str = format!("{}", &to.to_string());

            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            #[cfg(debug_assertions)]
            let _c_str1 = format!("{:?}", &c);

            let c = c
                .set_existential_definition(
                    exst,
                    Type::f(Type::Existential(a1n), Type::Existential(a2n)),
                )
                .add_before_existential(exst, a2)
                .add_before_existential(exst, a1);

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);

            let pred1_c = instantiate_l(c, a1n, from.as_ref(), type_map)?;
            let to_subst = pred1_c.substitute(to.as_ref());

            #[cfg(debug_assertions)]
            let _to_subst_str = to_subst.to_string();

            let pred2 = instantiate_r(pred1_c, a2n, &to_subst, type_map)?;
            Ok(pred2)
        }

        // InstRAllL
        Type::Forall(var, t) => {
            let next_ext = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Marker(next_ext))
                .append(ContextItem::Existential(next_ext, None));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let t = t.substitute_type_variable(var, &Type::Existential(next_ext))?;
            let pred1 = instantiate_l(c, exst, &t, type_map)?;
            Ok(pred1.get_before_item(ContextItem::Marker(next_ext)))
        }

        _ => {
            if !a.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            if let Some(existential) = c.get_existential(exst) {
                if let Some(existential_type_assignment) = existential {
                    subtype(c.clone(), &existential_type_assignment, &a.clone(), type_map)?;
                }
            }

            // InstRSolve
            Ok(c.set_existential_definition(exst, a.clone()))
        }
    }
}

// "Γ ⊢ e ⇒ A ⊣ ∆: Under input context Γ, e synthesizes output type A, with output context ∆"
fn synthesize_type(c: Context, ast: &AST, expr: usize, type_map: &TypeMap) -> Result<(Type, Context), TypeError> {
    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    let node = ast.get(expr);

    match node.t {
        // Var
        ASTNodeType::Identifier => {
            let var = node.get_value();

            #[cfg(debug_assertions)]
            let _var_str = var.clone();
            match c.get_type_assignment(&var) {
                Some(t) => Ok((t?, c)),
                None => unreachable!("Unbound type variable"),
            }
        }

        ASTNodeType::Pair => {
            let expr1 = ast.get_first(expr);
            let expr2 = ast.get_second(expr);
            let (expr1t, c) = synthesize_type(c, ast, expr1, type_map)?;
            let (expr2t, c) = synthesize_type(c, ast, expr2, type_map)?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            Ok((Type::pr(expr1t, expr2t), c))
        }

        ASTNodeType::Literal => Ok((node.get_lit_type(), c)),

        // ->I=>
        ASTNodeType::Abstraction => {
            let next_exst = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Existential(next_exst, None))
                .append(ContextItem::Existential(next_exst + 1, None));

            let c = if let Some(t) = &ast.get(ast.get_abstr_var(expr)).type_assignment {
                c.set_existential_definition(next_exst, t.clone())
            } else {
                c
            };

            let (c, before) = recurse_add_to_context(c, &Type::Existential(next_exst), ast, ast.get_abstr_var(expr))?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let c = check_type(
                c,
                &Type::Existential(next_exst + 1),
                ast,
                ast.get_abstr_expr(expr),
                type_map
            )?;

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);
            #[cfg(debug_assertions)]
            let _x = 1 + 1;

            let abst_type = Type::f(
                Type::Existential(next_exst),
                Type::Existential(next_exst + 1),
            );
            let c = c.get_before_assignment(before);

            #[cfg(debug_assertions)]
            let _c_str3 = format!("{:?}", &c);

            #[cfg(debug_assertions)]
            let _abst_type_str = format!("{}", &abst_type.to_string());

            Ok((abst_type, c))
        }

        // ->E
        ASTNodeType::Application => {
            let lhs = ast.get_func(expr);
            let rhs = ast.get_arg(expr);

            let (f_type, f_c) = synthesize_type(c, ast, lhs, type_map)?;

            #[cfg(debug_assertions)]
            let _f_c_str = format!("{:?}", &f_c);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();

            let f_type = f_c.substitute(&f_type);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();
            synthesize_app_type(f_c, &f_type, ast, rhs, type_map)
        }

        _ => unreachable!("Non expression"),
    }
}

// "Γ ⊢ A • e ⇒⇒ C ⊣ ∆: Under input context Γ, applying a function of type A to e synthesizes type C, with output context ∆"
fn synthesize_app_type(
    c: Context,
    applied_type: &Type,
    ast: &AST,
    expr: usize,
    type_map: &TypeMap
) -> Result<(Type, Context), TypeError> {
    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    #[cfg(debug_assertions)]
    let _applied_type = applied_type.to_string();

    match applied_type {
        // Forall App
        Type::Forall(var, t) => {
            let new_c = c.append(ContextItem::Existential(
                c.get_next_existential_identifier(),
                None,
            ));

            let a_subst = match t.substitute_type_variable(
                &var.clone(),
                &Type::Existential(c.get_next_existential_identifier()),
            ) {
                Ok(t) => t,
                Err(s) => {
                    panic!("Failed to substitute in forall app: {}", s)
                }
            };
            synthesize_app_type(new_c, &a_subst, ast, expr, type_map)
        }

        // -> App
        Type::Function(from, to) => {
            let pred = check_type(c, &from, ast, expr, type_map)?;

            Ok((to.as_ref().clone(), pred))
        }

        Type::Existential(var) => {
            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            let c = c
                .add_before_existential(*var, a2)
                .add_before_existential(*var, a1);
            let a1t = Type::Existential(a1n);
            let a2t = Type::Existential(a2n);
            let c = c.set_existential_definition(*var, Type::f(a1t, a2t.clone()));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let c = check_type(c, &a2t, ast, expr, type_map)?;

            Ok((a2t.clone(), c))
        }

        _ => Err(type_error("App synthesis error".to_string(), ast, expr)),
    }
}

fn recurse_add_to_context(c: Context, expected: &Type, ast: &AST, expr: usize) -> Result<(Context, String), TypeError>  {
    let pn = ast.get(expr);
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _expr_str = format!("{}", ast.to_string_sugar(expr, false));
    #[cfg(debug_assertions)]
    let _expected_str = format!("{}", &expected.to_string());

    match (expected, &pn.t) {
        (_, ASTNodeType::Identifier) => {
            let mut var_name = ast.get(expr).get_value();
            if var_name.starts_with("_") {
                var_name = c.get_next_placeholder_assignvar();
            }
            let new_ass = ContextItem::TypeAssignment(var_name.clone(), Ok(expected.clone()));
            Ok((c.append(new_ass.clone()), var_name))
        }
        (Type::Product(pt1, pt2), ASTNodeType::Pair) => {
            let pv1 = ast.get_first(expr);
            let pv2 = ast.get_second(expr);
            let (c, before) = recurse_add_to_context(c, pt1, &ast, pv1)?;
            let (c, _) = recurse_add_to_context(c, pt2, &ast, pv2)?;
            Ok((c, before))
        }
        (Type::Existential(e), ASTNodeType::Pair) => {
            let pv1 = ast.get_first(expr);
            let pv2 = ast.get_second(expr);
            let pt1 = c.get_next_existential_identifier();
            let pt2 = c.get_next_existential_identifier() + 1;

            let c = c.add_before_existential(*e, ContextItem::Existential(pt2, None));
            let c = c.add_before_existential(*e, ContextItem::Existential(pt1, None));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let (c, before)  = recurse_add_to_context(c, &Type::Existential(pt1), &ast, pv1)?;
            let (c, _) = recurse_add_to_context(c, &Type::Existential(pt2), &ast, pv2)?;

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);

            let c= c.set_existential_definition(*e, Type::pr(Type::Existential(pt1),Type::Existential(pt2)));
            Ok((c, before))
        }
        _ => Err(type_error("recurse add issue".to_string(), ast, expr)),
    }
}

// "Γ ⊢ e ⇐ A ⊣ ∆: Under input context Γ, e checks against input type A, with output context ∆"
fn check_type(c: Context, expected: &Type, ast: &AST, expr: usize, type_map: &TypeMap) -> Result<Context, TypeError> {
    let node = ast.get(expr);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    #[cfg(debug_assertions)]
    let _expected_type_str = expected.to_string();

    match (expected, &node.t) {
        // Unit always checks
        (Type::Unit, _) => Ok(c),

        // Forall Introduction
        (Type::Forall(var, t), _) => {
            let pred = check_type(
                c.append(ContextItem::TypeVariable(var.clone())),
                t,
                ast,
                expr,
                type_map
            )?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        // Arrow introduction
        (Type::Function(from, to), ASTNodeType::Abstraction) => {
            let var = ast.get_abstr_var(expr);

            let (c, before) = recurse_add_to_context(c, from, &ast, var)?;

            let pred = check_type(c, to, ast, ast.get_abstr_expr(expr), type_map)?;
            Ok(pred.get_before_assignment(before))
        }

        (Type::Product(pt1, pt2), ASTNodeType::Pair) => {
            let pair1 = check_type(c, pt1, ast, ast.get_first(expr), type_map)?;
            let pair2 = check_type(pair1, pt2, ast, ast.get_second(expr), type_map)?;

            Ok(pair2)
        }

        // Sub
        _ => {
            let (synth_t, c) = synthesize_type(c, ast, expr, type_map)?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            #[cfg(debug_assertions)]
            let _synth_t_str = synth_t.to_string();

            let a = c.substitute(&synth_t);
            let b = c.substitute(&expected);

            let st = subtype(c, &a, &b, type_map);

            match st {
                Ok(new_c) => {
                    #[cfg(debug_assertions)]
                    let _c_str = format!("{:?}", &new_c);

                    #[cfg(debug_assertions)]
                    let _x = 1 + 1;

                    Ok(new_c)
                }
                Err(e) => Err(type_error(
                    format!(
                        "Cannot figure out how {} could be subtype of {}: {}",
                        a.to_string(),
                        b.to_string(),
                        e
                    ),
                    ast,
                    expr,
                )),
            }
        }
    }
}

pub fn typecheck_tl_expr(expected: &Type, ast: &AST, expr: usize) -> Result<(), TypeError> {
    match check_type(
        Context::from_labels(&KnownTypeLabelTable::new()),
        expected,
        ast,
        expr,
        &TypeMap::new(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn infer_type_with_context(
    c: Context,
    ast: &AST,
    expr: usize,
    type_map: &TypeMap
) -> Result<(Type, Context), TypeError> {
    let (t, c) = synthesize_type(c, ast, expr, type_map)?;

    let t = c.substitute(&t);
    Ok((t, c))
}

#[cfg(test)]
pub fn infer_type(ast: &AST, expr: usize) -> Result<Type, TypeError> {
    let lt = KnownTypeLabelTable::new();
    let c = Context::from_labels(&lt);

    Ok(infer_type_with_context(c, ast, expr, &TypeMap::new())?.0.forall_ify())
}

pub fn infer_or_check_assignment_types(
    ast: &mut AST,
    module: usize,
    lt : &mut KnownTypeLabelTable,
    type_map: &TypeMap
) -> Result<(), TypeError> {
    let mut c = Context::from_labels(&lt);

    for assign_var in &ast.get_assignee_names(module) {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &c);

        let assign = ast.get_assign_to(module, assign_var.clone()).unwrap();

        #[cfg(debug_assertions)]
        let _assign_str = format!("{}", ast.to_string_sugar(assign, false));

        let assign_expr = ast.get_assign_exp(assign);

        let type_of_assignment = match &ast.get(assign).type_assignment {
            Some(type_assignment) => {
                c = c.append(ContextItem::TypeAssignment(
                    assign_var.clone(),
                    Ok(type_assignment.clone()),
                ));
                c = check_type(c, &type_assignment, ast, assign_expr, type_map)?;
                type_assignment.clone()
            }
            None => {
                c = c.append(ContextItem::TypeAssignment(
                    assign_var.clone(),
                    Err(type_error(format!("Cannot infer type of expression containing recursive call. Assign a type to label '{}'", &assign_var), ast, assign_expr)),
                ));
                let (t, new_c) = infer_type_with_context(c.clone(), &ast, assign_expr, type_map)?;
                let t = t.forall_ify();
                c = new_c.assigns_only().remove_assignment(assign_var).append(
                    ContextItem::TypeAssignment(assign_var.clone(), Ok(t.clone())),
                );
                ast.set_assignment_type(assign, t.clone());
                t
            }
        };

        lt.add(assign_var.clone(), type_of_assignment.clone());
    }

    Ok(())
}
