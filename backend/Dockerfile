FROM rust:latest as builder
WORKDIR /usr/src/backend
COPY . .
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    clang \
    && cargo build --release \
    && rm -rf /var/lib/apt/lists/*
FROM debian:bookworm-slim
COPY --from=builder /usr/src/backend/target/release/backend /usr/local/bin/backend
CMD ["backend"]