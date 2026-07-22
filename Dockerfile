# ==========================================
# STAGE 1: Build binary using latest official Rust image
# ==========================================
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache cargo dependencies build
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/backend_rust*

# Copy actual source code
COPY src ./src

# Build release binary
RUN cargo build --release

# ==========================================
# STAGE 2: Minimal runtime image
# ==========================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install SSL certificates & runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /usr/src/app/target/release/backend-rust /usr/local/bin/backend-rust

# Expose server port
EXPOSE 5000

# Set environment defaults
ENV PORT=5000 \
    RUST_LOG=info

# Run the binary
CMD ["/usr/local/bin/backend-rust"]
