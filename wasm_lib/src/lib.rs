mod utils;

use sfl_lib::{
    find_redex_contraction_pairs, infer_or_check_assignment_types, ASTNode, ASTNodeType,
    LabelTable, Parser, RCPair, AST,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RawASTInfo {
    pub ast: *mut AST,
    pub lt: *mut LabelTable,
}

#[wasm_bindgen]
pub struct RawRC {
    pub from_str: *mut String,
    pub to_str: *mut String,
    pub(crate) redex: *mut RCPair,
}

impl RawRC {
    pub unsafe fn free(&self) {
        if !&self.from_str.is_null() {
            drop(Box::from_raw(*&self.from_str as *mut String));
        }
        if !&self.to_str.is_null() {
            drop(Box::from_raw(*&self.to_str as *mut String));
        }
        if !&self.redex.is_null() {
            drop(Box::from_raw(*&self.redex));
        }
    }
}

#[wasm_bindgen]
pub unsafe fn get_rc_from(rc: &RawRC) -> String {
    (&*rc.from_str).clone()
}

#[wasm_bindgen]
pub unsafe fn get_rc_to(rc: &RawRC) -> String {
    (&*rc.to_str).clone()
}

#[wasm_bindgen]
pub unsafe fn pick_rc_and_free(info: &mut RawASTInfo, rcs: Vec<RawRC>, index: usize) {
    let ast = &mut *info.ast;
    let mut rust_rcs = vec![];
    for i in 0..rcs.len() {
        rust_rcs.push(&*rcs[i].redex);
    }
    ast.do_rc_subst_and_identical_rcs_borrowed(&*rcs[index].redex, &rust_rcs);
    for i in 0..rcs.len() {
        if i != index {
            rcs[i].free();
        }
    }
}

#[wasm_bindgen]
pub unsafe fn get_redexes(info: &RawASTInfo) -> Vec<RawRC> {
    let info = info;
    let ast = &mut *info.ast;
    let lt = &*info.lt;
    let module = ast.root;
    let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);
    let mut rcs = vec![];
    for rc in find_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt) {
        let from_str = Box::into_raw(Box::new(ast.to_string_sugar(rc.0, false).clone()));
        let to_string = Box::into_raw(Box::new(rc.1.to_string_sugar(rc.1.root, false).clone()));
        rcs.push(RawRC {
            from_str: from_str,
            to_str: to_string,
            redex: Box::into_raw(Box::new(rc)),
        });
    }
    rcs
}

#[wasm_bindgen]
pub fn parse(str: &str) -> Result<RawASTInfo, String> {
    let mut ast = match Parser::from_string(str.to_string()).parse_module() {
        Ok(ast) => ast,
        Err(e) => return Err(format!("{:?}", e)),
    };
    let module = ast.root;
    let lt = match infer_or_check_assignment_types(&mut ast, module) {
        Ok(lt) => lt,
        Err(e) => return Err(format!("{:?}", e)),
    };
    Ok(RawASTInfo {
        ast: Box::into_raw(Box::new(ast)),
        lt: Box::into_raw(Box::new(lt)),
    })
}

#[wasm_bindgen]
pub unsafe fn to_string(info: &RawASTInfo) -> String {
    let info = info;
    let ast = &*info.ast;
    ast.to_string_sugar(ast.root, true)
}
