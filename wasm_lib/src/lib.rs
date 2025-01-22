mod utils;

use sfl_lib::{find_redex_contraction_pairs, infer_or_check_assignment_types, ASTNode, ASTNodeType, LabelTable, Parser, RCPair, AST};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, hello-wasm!");
}

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello from Rust!".to_string()
}


#[wasm_bindgen]
pub struct RawASTInfo {
    pub ast: *mut AST,
    pub lt: *mut LabelTable,
}

#[wasm_bindgen]
pub unsafe fn get_redexes(info: RawASTInfo) -> *mut Vec<RCPair> {
    let info = info;
    let ast = &mut *info.ast;
    let lt = &*info.lt;
    let module = ast.root;
    let main_assign = ast.get_assign_to(module, "main".to_string()).unwrap();
    let main_expr = ast.get_assign_exp(main_assign);
    let rc = find_redex_contraction_pairs(&ast, Some(ast.root), main_expr, &lt);
    Box::into_raw(Box::new(rc))
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
pub unsafe fn to_string(info: RawASTInfo) -> String {
    let info = info;
    let ast = &*info.ast;
    ast.to_string_sugar(ast.root, true)
}