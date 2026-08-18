# Multi-stage ultra-lightweight build (<20MB final image)
FROM rust:1.82-alpine AS builder

RUN apk add --no-cache musl-dev

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
