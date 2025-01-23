FROM kiranisturt/wasm_pack:1.81 AS wasm-builder

COPY wasm_lib /build/wasm_lib
COPY sfl_lib /build/sfl_lib

ENV PATH="/root/.cargo/bin:${PATH}"

# Build the wasm library
WORKDIR /build/wasm_lib
RUN wasm-pack build --target web

# Build the frontend
FROM node:18-alpine AS frontend-builder

COPY --from=wasm-builder /build/wasm_lib/pkg /build/wasm_lib/pkg
COPY frontend /build/frontend

WORKDIR /build/frontend
# Remove node modules if they exist
RUN rm -rf node_modules
RUN npm ci
RUN npm run build