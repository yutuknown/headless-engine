# Multi-stage ultra-lightweight build (<20MB final image)
# Use latest stable Rust to ensure edition2024 dependency support
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl --bin headless-engine

# Runtime container
FROM alpine:3.20

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/headless-engine /usr/local/bin/headless-engine

EXPOSE 9222

ENTRYPOINT ["/usr/local/bin/headless-engine"]
CMD ["--stdio"]
