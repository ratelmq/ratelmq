# ===== builder =====
FROM rust:1.56-slim-buster as builder

WORKDIR /usr/src/ratelmq

RUN apt-get update && \
    apt-get install -y git && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p ./src && \
    echo "fn main() {print!(\"foo\");}" > ./src/main.rs

COPY ./Cargo.toml ./
# cache dependencies
RUN cargo build --release

COPY . ./
RUN cargo build --release

# ===== actual image =====
FROM debian:buster-slim
LABEL maintainer="Wojciech Wilk w.wilk@metasoftworks.com"

RUN mkdir -p /etc/ratelmq

COPY ./config/ratelmq.toml /etc/ratelmq/ratelmq.toml

COPY --from=builder /usr/src/ratelmq/target/release/ratelmq /ratelmq

EXPOSE 1883

ENV RUST_LOG=INFO

ENTRYPOINT ["/ratelmq"]
