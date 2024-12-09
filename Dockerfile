FROM rust:1.80 AS wasm-builder

COPY wasm-lib /build/wasm-lib
COPY sfl_compiler /build/sfl_compiler

# Install wasm-pack
RUN cargo install wasm-pack

ENV PATH="/root/.cargo/bin:${PATH}"

# Build the wasm library
WORKDIR /build/wasm-lib
RUN wasm-pack build --target web

# Build the frontend
FROM node:18-alpine AS frontend-builder

COPY --from=wasm-builder /build/wasm-lib/pkg /build/wasm-lib/pkg
COPY . /build

WORKDIR /build
RUN npm install
RUN npm run build