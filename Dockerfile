# ---- Builder stage ----------------------------------------------------------
FROM rust:1.81-bookworm AS builder
WORKDIR /app

# Leverage Docker layer caching: copy manifests first
COPY Cargo.toml Cargo.lock* ./
# Create a dummy src to cache deps even if your real src changes
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Pre-build dependencies
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release || true

# Now copy real sources and build
COPY src ./src
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

# ---- Runtime stage ----------------------------------------------------------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Minimal runtime deps for TLS
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Non-root user
RUN useradd -u 10001 -m appuser

# Copy binary
COPY --from=builder /app/target/release/ru-ip-dump /usr/local/bin/ru-ip-dump

# Working data dir (bind-mount here)
VOLUME ["/data"]
WORKDIR /data

USER appuser

# Default: run and write outputs to /data
ENTRYPOINT ["/usr/local/bin/ru-ip-dump"]