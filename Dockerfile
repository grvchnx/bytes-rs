# Multi-stage Dockerfile for bytes-rs
FROM node:20-bookworm AS builder

# Install Rust toolchain
RUN apt-get update && apt-get install -y curl build-essential && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy dependency files
COPY package.json package-lock.json Cargo.toml Cargo.lock ./

# Install npm dependencies
RUN npm ci

# Copy source code and tests
COPY . .

# Build Rust release binary & N-API module
RUN npm run build

# Run Rust unit tests & original JS mocha test suite
RUN npm run test-rust
RUN npm test

# Run bench test to ensure runnable artifact
RUN node bench/bench.js

CMD ["npm", "test"]
