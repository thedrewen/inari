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

RUN cargo build --release

# ---

FROM debian:trixie-slim AS runner

WORKDIR /app

COPY --from=node-builder ./dist/inari/browser ./static
COPY --from=rust-builder ./target/release/inari .

CMD ["./inari"]