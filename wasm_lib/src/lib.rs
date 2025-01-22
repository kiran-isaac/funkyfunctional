mod utils;

use std::rc;

use sfl_lib::{
    find_redex_contraction_pairs, infer_or_check_assignment_types, ASTNode, ASTNodeType,
    LabelTable, Parser, RCPair, AST,
};
use wasm_bindgen::prelude::*;
use web_sys::console::log;

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

pub unsafe fn get_rc_from(rc: &RawRC) -> String {
    (&*rc.from_str).clone()
}

pub unsafe fn get_rc_to(rc: &RawRC) -> String {
    (&*rc.to_str).clone()
}

#[wasm_bindgen]
pub unsafe fn get_rcs_from(rcs: *mut Vec<RawRC>, rc: usize) -> String {
    let rcs = &*rcs;
    get_rc_from(&rcs[rc])
}

#[wasm_bindgen]
pub unsafe fn get_rcs_to(rcs: *mut Vec<RawRC>, rc: usize) -> String {
    let rcs = &*rcs;
    get_rc_to(&rcs[rc])
}

#[wasm_bindgen]
pub unsafe fn get_rcs_len(rcs: *mut Vec<RawRC>) -> usize {
    let rcs = &*rcs;
    rcs.len()
}

#[wasm_bindgen]
pub unsafe fn pick_rc_and_free(info: &mut RawASTInfo, rcs: *mut Vec<RawRC>, to_subst: usize) {
    let rcs = &*rcs;

    let ast = &mut *info.ast;
    let mut rust_rcs = vec![];

    log!("len: {}\nchosen: {}", rcs.len(), to_subst);    
    
    for rc in rcs {
        // log!("rc: {}", ast.rc_to_str(&*rc.redex));
        rust_rcs.push(&*rc.redex);
    }

    ast.do_rc_subst_and_identical_rcs_borrowed(&*rcs[to_subst].redex, &rust_rcs);

    for rc in rcs {
        log!("{}", ast.rc_to_str(&*rc.redex));
        rc.free();
    }
}

#[wasm_bindgen]
pub unsafe fn get_laziest(info: &RawASTInfo, rcs: *mut Vec<RawRC>) -> usize {
    let info = info;
    let ast = &mut *info.ast;
    let rcs = &*rcs;

    let module = ast.root;
    let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);

    let mut rust_rcs = vec![];
    
    for rc in rcs {
        // log!("rc: {}", ast.rc_to_str(&*rc.redex));
        rust_rcs.push(&*rc.redex);
    }

    let laziest = ast.get_laziest_rc_borrowed(main_expr, &rust_rcs).unwrap();

    for (i, rc) in rust_rcs.iter().enumerate() {
        if rc.0 == laziest.0 {
            return i
        }
    }

    unreachable!("FAILED TO GET LAZIEST");
}

#[wasm_bindgen]
pub unsafe fn get_redexes(info: &RawASTInfo) -> *mut Vec<RawRC> {
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
    Box::into_raw(Box::new(rcs))
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
