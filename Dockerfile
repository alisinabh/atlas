ARG DEBIAN_RELEASE=trixie

FROM rust:1.99.1-$DEBIAN_RELEASE AS builder

WORKDIR /app

COPY ./src ./src
COPY Cargo.* ./

RUN cargo build --release

FROM debian:$DEBIAN_RELEASE-slim AS runtime

# Apply Debian security updates newer than the base image, so a stale base tag
# doesn't ship known CVEs. Build with `--pull --no-cache-filter runtime` to refresh.
RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/atlas /usr/local/bin/

RUN mkdir -p /opt/atlas/db && chown nobody:root /opt/atlas/db

USER nobody

ENV DB_PATH=/opt/atlas/db

CMD ["atlas"]
