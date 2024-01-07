FROM rust:latest as builder

ARG API_URL
ENV API_URL=$API_URL

WORKDIR /usr/src/app
COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/spellbook ./spellbook 

RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    cargo install trunk && cd web && trunk build --release

FROM debian:bookworm
RUN apt-get update
RUN apt-get install -y openssl libssl-dev ca-certificates


RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY --from=builder /usr/src/app/spellbook /app/spellbook
COPY --from=builder /usr/src/app/dist /app/dist/

EXPOSE 8080

CMD ./spellbook
