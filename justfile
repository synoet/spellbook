api: 
  cargo run --bin spellbook

web:
  cd web && API_URL=http://localhost:8080 trunk serve --open

deploy:
  flyctl deploy

build-docker:
  docker build -t spellbook-api .

run-docker:
  docker run -p 8080:8080 -it --env-file=./api/.env spellbook-api
