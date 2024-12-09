# SFL Explorer
A step by step evaluator for a Simple Functional Language

## Architecture
The submodule "sfl_compiler" contains the compiler for the Simple Functional Language (SFL). The compiler is written in Rust, and building it will generate a binary that can be used to compile SFL programs. It includes a lib, which is included by the wasm_lib rust module, which compiles to a wasm module that can be used in the browser. The wasm-lib rust module is compiled by wasm-pack. 

## Build
Dependencies:
- node v18
- cargo, rustc etc. Install through rustup to make life easier: https://rustup.rs/
- wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)

To build the project, run the following command:
```bash
$ npm run build
```
This will build the wasm module and the web app. The web app will be built in the `build` directory.

See the Readme in the sfl_compiler directory for more information on how to build the compiler standalone.