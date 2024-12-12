FROM rust:1.80 AS wasm-builder

COPY wasm_lib /build/wasm_lib
COPY sfl_lib /build/sfl_lib

# Install wasm-pack
RUN cargo install wasm-pack

ENV PATH="/root/.cargo/bin:${PATH}"

# Build the wasm library
WORKDIR /build/wasm_lib
RUN wasm-pack build --target web

# Build the frontend
FROM node:18-alpine AS frontend-builder

COPY --from=wasm-builder /build/wasm_lib/pkg /build/wasm_lib/pkg
COPY . /build

WORKDIR /build
RUN npm install
RUN npm run build