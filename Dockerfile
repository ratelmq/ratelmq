FROM rust:1.50-slim-buster as builder

WORKDIR /usr/src/ratelmq

RUN mkdir -p ./src && echo "fn main() {print!(\"foo\");}" > ./src/main.rs
COPY ./Cargo.toml ./Cargo.toml
# cache dependencies
RUN cargo build --release

COPY ./src ./src
# Force rebuild
RUN touch -t 200001010000 ./target/release/ratelmq && \
    cargo build --release

# ===== actual image =====
FROM debian:buster-slim
LABEL maintainer="Wojciech Wilk w.wilk@metasoftworks.com"

COPY --from=builder /usr/src/ratelmq/target/release/ratelmq /ratelmq

EXPOSE 1883

ENV RUST_LOG=INFO

CMD ["/ratelmq"]
