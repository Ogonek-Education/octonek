# Build stage
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create dummy project to cache dependencies
RUN cargo new --bin ogonek-gh
WORKDIR /app/ogonek-gh
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs && rm -f target/release/deps/ogonek_gh*

# Copy actual source and build
COPY src ./src
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/ogonek-gh/target/release/ogonek-gh .

# Set the binary as the entrypoint
ENTRYPOINT ["./ogonek-gh"]
