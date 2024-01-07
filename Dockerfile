FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/spellbook ./spellbook 

COPY ./dist ./dist

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
