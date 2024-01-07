FROM rust:slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/spellbook ./spellbook

FROM debian:bookworm-slim AS final

RUN apt-get update && apt-get install -y openssl

WORKDIR /app

COPY --from=builder /usr/src/app/spellbook /app/spellbook

EXPOSE 8000
CMD ./spellbook
