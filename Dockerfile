# syntax=docker/dockerfile:1

### ---- Stage 1: Build Tooka ----
FROM rust:1.87 AS builder

# Install build tools
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/tooka

# Accept TOOKA_REF arg (branch or tag name)
ARG TOOKA_REF=main

# Clone repo and checkout ref
RUN git clone --depth 1 --branch ${TOOKA_REF} https://github.com/Benji377/tooka.git .

# Build Tooka using musl
RUN cargo build -p tooka-cli --release --target x86_64-unknown-linux-musl

### ---- Stage 2: Runtime Container ----
FROM alpine:latest

# Install minimal runtime dependencies
RUN apk add --no-cache ca-certificates bash

# Copy static binary
COPY --from=builder /usr/src/tooka/target/x86_64-unknown-linux-musl/release/tooka /usr/local/bin/tooka

# Set entrypoint to Tooka
ENTRYPOINT ["tooka"]
CMD ["--help"]
