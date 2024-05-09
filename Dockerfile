ARG DEBIAN_RELEASE=bookworm

FROM rust:1-$DEBIAN_RELEASE AS builder

WORKDIR /app

COPY ./src ./src
COPY Cargo.* ./

RUN cargo build --release

FROM debian:$DEBIAN_RELEASE-slim

COPY --from=builder /app/target/release/atlas /usr/local/bin/

RUN mkdir -p /opt/atlas/db && chown nobody:root /opt/atlas/db

USER nobody

ENV DB_PATH=/opt/atlas/db

CMD ["atlas"]
