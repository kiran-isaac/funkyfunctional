use crate::{functions::LabelTable, ASTNodeType, AST};

use super::{Type, TypeError};

#[derive(Clone, PartialEq, Eq)]
enum ContextItem {
    TypeVariable(usize),
    TypeAssignment(String, Type),
    Existential(usize, Option<Type>),
    Marker(usize),
}

impl std::fmt::Debug for ContextItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextItem::TypeVariable(v) => write!(f, "{}", Type::TypeVariable(*v).to_string()),
            ContextItem::Existential(v, ass) => match ass {
                Some(t) => write!(f, "{}:{}", Type::Existential(*v).to_string(), t.to_string()),
                None => write!(f, "{}", Type::Existential(*v).to_string()),
            },
            ContextItem::Marker(v) => write!(f, "|{}|", Type::Existential(*v).to_string()),
            ContextItem::TypeAssignment(name, t) => write!(f, "{}:{}", name, t.to_string()),
        }
    }
}

#[derive(Clone)]
struct Context {
    vec: Vec<ContextItem>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inbuilt_map = LabelTable::new().get_type_map();
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
    fn from_labels(labels: &LabelTable) -> Self {
        let mut vec = vec![];

        for (k, v) in labels.get_type_map() {
            vec.push(ContextItem::TypeAssignment(k.clone(), v.clone()));
        }

        Self { vec }
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

        Self { vec: new_v }
    }

    fn append(&self, item: ContextItem) -> Self {
        let mut new = self.clone();

        new.vec.push(item);
        new
    }

    fn get_before_item(&self, item: ContextItem) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            if i == &item {
                break;
            }

            new_v.push(i.clone());
        }

        Self { vec: new_v }
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

        let new_s = Self { vec: new_v };

        #[cfg(debug_assertions)]
        let _new_s_str = format!("{:?}", &new_s);
        new_s
    }

    fn get_type_assignment(&self, var: &str) -> Option<Type> {
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

        for i in &self.vec {
            match i {
                ContextItem::Existential(e, _) => {
                    if *e == existential {
                        new_v.push(item.clone());
                    }
                }
                _ => {}
            }
            new_v.push(i.clone());
        }

        Self { vec: new_v }
    }

    fn set_existential_definition(&self, existential: usize, t: Type) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                // If this is another context that references the one being substituted then
                // Substitute this one too
                ContextItem::Existential(e, Some(Type::Existential(e2))) => {
                    if *e2 == existential {
                        new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                        continue;
                    }
                }
                ContextItem::Existential(e, _) => {
                    if *e == existential {
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

        Self { vec: new_v }
    }

    fn get_next_existential_identifier(&self) -> usize {
        let mut max = 0;

        for i in &self.vec {
            match i {
                ContextItem::Existential(n, _) => {
                    max = std::cmp::max(*n, max);
                }
                _ => {}
            }
        }

        max + 1
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
                    unimplemented!()
                }
            },
            Type::Function(from, to) => Type::Function(
                Box::new(self.substitute(from.as_ref())),
                Box::new(self.substitute(to.as_ref())),
            ),
            Type::Forall(var, t) => Type::Forall(*var, Box::new(self.substitute(t.as_ref()))),
            _ => t.clone(),
        }
    }
}

fn type_error(msg: String, ast: &AST, expr: usize) -> TypeError {
    let n = ast.get(expr);
    return TypeError {
        e: msg,
        col: n.col,
        line: n.line,
    };
}

fn subtype(c: Context, a: &Type, b: &Type) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _a_str = a.to_string();
    #[cfg(debug_assertions)]
    let _b_str = b.to_string();

    match (a, b) {
        (Type::Existential(ex1), Type::Existential(ex2)) => {
            if ex1 == ex2 {
                Ok(c)
            } else {
                instantiate_l(c, *ex1, b)
            }
        }
        // <:InstantiateL
        (Type::Existential(ex), _) => {
            assert!(!b.contains_existential(*ex));

            instantiate_l(c, *ex, b)
        }

        // <:InstantiateR
        (_, Type::Existential(ex)) => {
            assert!(!a.contains_existential(*ex));

            instantiate_r(c, *ex, a)
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
                Err(format!("{:?} is not a subtype of {:?}", a, b))
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

            let new_body = t.substitute_type_variable(*var, &Type::Existential(exst))?;
            let pred = subtype(c, &new_body, b)?;
            Ok(pred.get_before_item(ContextItem::Marker(exst)))
        }

        // <:ForallR
        (_, Type::Forall(var, t)) => {
            let c = c.append(ContextItem::TypeVariable(*var));
            let pred = subtype(c, a, t.as_ref())?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(*var)))
        }

        // <:->
        (Type::Function(a1, a2), Type::Function(b1, b2)) => {
            let pred1 = subtype(c, b1.as_ref(), a1)?;
            let a2 = &pred1.substitute(a2);
            let b2 = &pred1.substitute(b2);
            let pred2 = subtype(pred1, a2, b2)?;
            Ok(pred2)
        }

        _ => Err("Subtype failiure".to_string()),
    }
}

fn instantiate_l(c: Context, exst: usize, b: &Type) -> Result<Context, String> {
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

            let pred1_c = instantiate_r(c, exst, from.as_ref())?;
            let to_subst = pred1_c.substitute(to);

            #[cfg(debug_assertions)]
            let _to_subst_str = format!("{:?}", &to_subst);

            let pred2 = instantiate_l(pred1_c, a2n, &to_subst)?;
            Ok(pred2)
        }

        // InstLAllR
        Type::Forall(var, t) => {
            let new_c = c.append(ContextItem::TypeVariable(*var));
            let pred = instantiate_l(new_c, exst, t.as_ref())?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(*var)))
        }

        _ => {
            if !b.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            // InstLSolve
            Ok(c.set_existential_definition(exst, b.clone()))
        }
    }
}

fn instantiate_r(c: Context, exst: usize, a: &Type) -> Result<Context, String> {
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

            let pred1_c = instantiate_l(c, a1n, from.as_ref())?;
            let to_subst = pred1_c.substitute(to.as_ref());

            #[cfg(debug_assertions)]
            let _to_subst_str = to_subst.to_string();

            let pred2 = instantiate_r(pred1_c, a2n, &to_subst)?;
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

            let t = t.substitute_type_variable(*var, &Type::Existential(next_ext))?;
            let pred1 = instantiate_l(c, exst, &t)?;
            Ok(pred1.get_before_item(ContextItem::Marker(next_ext)))
        }

        _ => {
            if !a.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            // InstRSolve
            Ok(c.set_existential_definition(exst, a.clone()))
        }
    }
}

// "Γ ⊢ e ⇒ A ⊣ ∆: Under input context Γ, e synthesizes output type A, with output context ∆"
fn synthesize_type(c: Context, ast: &AST, expr: usize) -> Result<(Type, Context), TypeError> {
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
                Some(t) => Ok((t, c)),
                None => Err(type_error("Unbound variable".to_string(), ast, expr)),
            }
        }

        ASTNodeType::Literal => Ok((node.get_lit_type(), c)),

        // ->I=>
        ASTNodeType::Abstraction => {
            let next_exst = c.get_next_existential_identifier();
            let abst_var = ast.get(ast.get_abstr_var(expr)).get_value();
            let c = c
                .append(ContextItem::Existential(next_exst, None))
                .append(ContextItem::Existential(next_exst + 1, None))
                .append(ContextItem::TypeAssignment(
                    abst_var.clone(),
                    Type::Existential(next_exst),
                ));

            let c = if let Some(t) = &ast.get(ast.get_abstr_var(expr)).type_assignment {
                c.set_existential_definition(next_exst, t.clone())
            } else {
                c
            };

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let c = check_type(
                c,
                &Type::Existential(next_exst + 1),
                ast,
                ast.get_abstr_exp(expr),
            )?;

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);
            #[cfg(debug_assertions)]
            let _x = 1 + 1;

            let abst_type = Type::f(
                Type::Existential(next_exst),
                Type::Existential(next_exst + 1),
            );
            let c = c.get_before_assignment(abst_var);

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

            let (f_type, f_c) = synthesize_type(c, ast, lhs)?;

            #[cfg(debug_assertions)]
            let _f_c_str = format!("{:?}", &f_c);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();

            let f_type = f_c.substitute(&f_type);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();
            synthesize_app_type(f_c, &f_type, ast, rhs)
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
                *var,
                &Type::Existential(c.get_next_existential_identifier()),
            ) {
                Ok(t) => t,
                Err(s) => {
                    return Err(type_error(
                        format!("Failed to substitute in forall app: {}", s),
                        ast,
                        expr,
                    ))
                }
            };
            synthesize_app_type(new_c, &a_subst, ast, expr)
        }

        // -> App
        Type::Function(from, to) => {
            let pred = check_type(c, &from, ast, expr)?;

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

            let c = check_type(c, &a2t, ast, expr)?;

            Ok((a2t.clone(), c))
        }

        _ => Err(type_error("App synthesis error".to_string(), ast, expr)),
    }
}

// "Γ ⊢ e ⇐ A ⊣ ∆: Under input context Γ, e checks against input type A, with output context ∆"
fn check_type(c: Context, expected: &Type, ast: &AST, expr: usize) -> Result<Context, TypeError> {
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
            let pred = check_type(c.append(ContextItem::TypeVariable(*var)), t, ast, expr)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(*var)))
        }

        // Arrow introduction
        (Type::Function(from, to), ASTNodeType::Abstraction) => {
            let var_name = ast.get(ast.get_abstr_var(expr)).get_value();

            let new_ass = ContextItem::TypeAssignment(var_name.clone(), from.as_ref().clone());
            let c = c.append(new_ass.clone());
            let pred = check_type(c, to, ast, ast.get_abstr_exp(expr))?;
            Ok(pred.get_before_item(new_ass))
        }

        // Sub
        _ => {
            let (synth_t, c) = synthesize_type(c, ast, expr)?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            #[cfg(debug_assertions)]
            let _synth_t_str = synth_t.to_string();

            let a = c.substitute(&synth_t);
            let b = c.substitute(&expected);

            let st = subtype(c, &a, &b);

            match st {
                Ok(new_c) => {
                    #[cfg(debug_assertions)]
                    let _c_str = format!("{:?}", &new_c);

                    #[cfg(debug_assertions)]
                    let _x = 1 + 1;

                    Ok(new_c)
                }
                Err(e) => Err(type_error(format!("Check sub error: {}", e), ast, expr)),
            }
        }
    }
}

pub fn typecheck_tl_expr(expected: &Type, ast: &AST, expr: usize) -> Result<(), TypeError> {
    match check_type(
        Context::from_labels(&LabelTable::new()),
        expected,
        ast,
        expr,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
pub fn typecheck_module(ast: &AST, module: usize) -> Result<LabelTable, TypeError> {
    let mut lt = LabelTable::new();
    let mut c = Context::from_labels(&lt);

    for assign_var in &ast.get_assignee_names(module) {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &c);

        let assign = ast.get_assign_to(module, assign_var.clone()).unwrap();

        #[cfg(debug_assertions)]
        let _assign_str = format!("{}", ast.to_string_sugar(assign, false));

        let assign_expr = ast.get_assign_exp(assign);
        let assign_type = ast.get(assign).type_assignment.clone().unwrap();
        c = c.assigns_only().append(ContextItem::TypeAssignment(
            assign_var.clone(),
            assign_type.clone(),
        ));
        c = check_type(c, &assign_type, ast, assign_expr)?;
        lt.add(assign_var.clone(), assign_type.clone());
    }

    Ok(lt)
}

pub fn infer_type(ast: &AST, expr: usize) -> Result<Type, TypeError> {
    let lt = LabelTable::new();
    let c = Context::from_labels(&lt);

    match synthesize_type(c, ast, expr) {
        Ok((t, c)) => {
            Ok(c.substitute(&t).forall_ify().settle_tvs())
        }
        Err(e) => Err(e),
    }
}
