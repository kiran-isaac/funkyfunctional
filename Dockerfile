FROM rust:1.80 AS wasm-builder

COPY wasm-lib /redexplore/wasm-lib
COPY sfl_compiler /redexplorer/sfl_compiler

# Install wasm-pack
RUN cargo install wasm-pack

ENV PATH="/root/.cargo/bin:${PATH}"

# Build the wasm library
WORKDIR /redexplore/wasm-lib
RUN wasm-pack build --target web

# Build the frontend
FROM node:18-alpine AS frontend-builder

COPY --from=wasm-builder /redexplore/wasm-lib/pkg /redexplore/wasm-lib/pkg
COPY . /redexplore

WORKDIR /redexplore
RUN npm install
RUN npm run build