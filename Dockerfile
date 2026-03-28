# ── Build stage ──
FROM rust:1.87-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

# ── Runtime stage ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/spine /usr/local/bin/spine

ENV SPINE_PORT=3000
ENV SPINE_DB_PATH=/data/spine.db
ENV SPINE_EMBED_URL=http://spine-embed:8000

EXPOSE 3000

VOLUME ["/data"]

CMD ["spine"]
