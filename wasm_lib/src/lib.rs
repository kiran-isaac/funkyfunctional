mod utils;

use sfl_lib::{ASTNode, ASTNodeType, Parser, AST};
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
pub fn parse(str: &str) -> *mut AST {
    let ast = Parser::from_string(str.to_string()).parse_module().unwrap();
    let b = unsafe {Box::into_raw(Box::new(ast))};
    b
}