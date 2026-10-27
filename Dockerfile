FROM node:26 AS node-builder

WORKDIR /app

COPY frontend/ .

RUN npm i && npm run build

# ---

FROM rust AS rust-builder

WORKDIR /app

COPY server/ .

RUN apt-get update && apt-get install -y python3 python3-pip ninja-build && \
  rustup component add rustfmt

ENV CARGO_TARGET_DIR=/build/target

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    cp /build/target/release/inari ./inari

# ---

FROM debian:trixie-slim AS runner

WORKDIR /app

COPY --from=node-builder /app/dist/inari/browser ./static
COPY --from=rust-builder /app/inari .

CMD ["./inari"]