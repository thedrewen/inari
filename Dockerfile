FROM debian:trixie-slim

WORKDIR /app

COPY target/ /app

CMD ["./inari"]