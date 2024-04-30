ARG DEBIAN_RELEASE=bookworm

FROM rust:1-$DEBIAN_RELEASE AS builder

WORKDIR /app

COPY ./src ./src
COPY Cargo.* ./

RUN cargo build --release

FROM debian:$DEBIAN_RELEASE-slim

RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/atlas /usr/local/bin/

USER nobody

CMD ["atlas"]
