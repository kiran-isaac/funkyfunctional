mod utils;

use sfl_lib::AST;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ASTWasmBinding {
    pub ast: AST
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, hello-wasm!");
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    sfl_lib::add(a as i64, b as i64) as i32
}

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello from Rust!".to_string()
}