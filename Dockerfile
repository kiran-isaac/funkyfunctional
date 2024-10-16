FROM node:18-alpine

COPY . /redexplore

# Install Rust through rustup
RUN apk add --no-cache cargo rust
RUN curl –proto ‘=https’ –tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm target and wasm-pack
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack

# Build the wasm library
WORKDIR /redexplore/wasm-lib
RUN wasm-pack build --target web

# Build the frontend
WORKDIR /redexplore
RUN npm install
RUN npm run build

# Copy the build to the final image
RUN cp -r /redexplore/build /build
RUN rm -rf /redexplore

# Serve the frontend
WORKDIR /build
RUN npm start