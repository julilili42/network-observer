FROM rust:1.85 AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y libpcap-dev
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY src ./src
RUN touch src/*.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpcap0.8 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/network_sniffer /usr/local/bin/
EXPOSE 3000
CMD ["network_sniffer"]