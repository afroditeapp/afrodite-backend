FROM rust:slim AS builder
WORKDIR /usr/src/afrodite-backend
COPY . .
RUN apt-get update && \
 apt-get install -y git build-essential pkg-config libssl-dev libsqlite3-dev libpq-dev && \
 rm -rf /var/lib/apt/lists/*
RUN cd crates/afrodite-backend && cargo install --path .

FROM debian:stable-slim
RUN apt-get update && \
 apt-get upgrade -y && \
 apt-get install -y ca-certificates libpq5 && \
 rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/afrodite-backend /usr/local/bin/afrodite-backend
ENTRYPOINT ["/usr/local/bin/afrodite-backend"]
