#[allow(unused)]
mod utils;

use sfl_lib::{
    find_all_redex_contraction_pairs, find_single_redex_contraction_pair,
    infer_or_check_assignment_types, KnownTypeLabelTable, Parser, RCPair, AST,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RawASTInfo {
    pub ast: *mut AST,
    pub lt: *mut KnownTypeLabelTable,
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
pub unsafe fn pick_rc_and_free(
    info: &mut RawASTInfo,
    rcs: *mut Vec<RawRC>,
    to_subst: usize,
) -> RawASTInfo {
    let rcs = &*rcs;

    let ast = &mut *info.ast;
    let lt = &*info.lt;
    let mut rust_rcs = vec![];

    for rc in rcs {
        rust_rcs.push(&*rc.redex);
    }

    let mut ast2 = ast.clone();
    ast2.do_rc_subst_and_identical_substs_borrowed(&*rcs[to_subst].redex);

    for rc in rcs {
        rc.free();
    }

    // clone to cleanup and remove orphan nodes
    let ast2 = ast2.clone_node(ast2.root);

    return RawASTInfo {
        ast: Box::into_raw(Box::new(ast2)),
        lt: Box::into_raw(Box::new(lt.clone())),
    };
}

#[wasm_bindgen]
pub unsafe fn get_all_redexes(info: &RawASTInfo) -> *mut Vec<RawRC> {
    let info = info;
    let ast = &mut *info.ast;
    let lt = &*info.lt;
    let module = ast.root;
    let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);
    let mut rcs_output: Vec<RawRC> = vec![];
    let rcs = find_all_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    for rc in ast.filter_identical_rcs(&rcs) {
        let from_str = Box::into_raw(Box::new(ast.to_string_sugar(rc.0, false).clone()));
        let to_string = Box::into_raw(Box::new(rc.1.to_string_sugar(rc.1.root, false).clone()));
        rcs_output.push(RawRC {
            from_str: from_str,
            to_str: to_string,
            redex: Box::into_raw(Box::new(rc)),
        });
    }
    Box::into_raw(Box::new(rcs_output))
}

#[wasm_bindgen]
pub unsafe fn get_one_redex(info: &RawASTInfo) -> *mut Vec<RawRC> {
    let info = info;
    let ast = &mut *info.ast;
    let lt = &*info.lt;
    let module = ast.root;
    let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);

    Box::into_raw(Box::new(
        if let Some(rc) = find_single_redex_contraction_pair(&ast, Some(ast.root), main_expr, &lt) {
            let from_str = Box::into_raw(Box::new(ast.to_string_sugar(rc.0, false).clone()));
            let to_string = Box::into_raw(Box::new(rc.1.to_string_sugar(rc.1.root, false).clone()));
            vec![RawRC {
                from_str: from_str,
                to_str: to_string,
                redex: Box::into_raw(Box::new(rc)),
            }]
        } else {
            vec![]
        },
    ))
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

#[wasm_bindgen]
pub unsafe fn main_to_string(info: &RawASTInfo) -> String {
    let info = info;
    let ast = &*info.ast;
    let main_assign = ast.get_assign_to(ast.root, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);
    ast.to_string_sugar(main_expr, true)
}

// #[wasm_bindgen]
// pub unsafe fn get_highlight_regions(
//     info: &RawASTInfo,
//     rcs: *mut Vec<RawRC>,
//     to_subst: usize,
// ) -> String {
//     let info = info;
//     let ast = &*info.ast;
//     let main_assign = ast.get_assign_to(ast.root, "main".to_string()).unwrap();
//     let main_expr = ast.get_assign_exp(main_assign);
//     let rc_expr = (&*(&*rcs)[to_subst].redex).0;

// }
