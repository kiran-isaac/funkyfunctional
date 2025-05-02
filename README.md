# SFL Explorer

## build instructions
### Dependencies
Older versions of the dependencies may work, but are not guaranteed to work.
- nodejs >= 22
- npm >= 9
- cargo, rustc >= 1.86.0
- wasm-pack >= 0.13.0. Install with `cargo install wasm-pack`.

### Build
build the wasm-lib using the following command:
```bash
cd wasm-lib
wasm-pack build --target web
```
This will build the rust code into a wasm package. 

Build the frontend. cd into the `frontend` directory and run the following commands:
```bash
npm install
npm run build
```
This will build the frontend and the rust code into a single package. The output will be in the `dist` directory.
