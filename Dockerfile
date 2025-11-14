FROM rustlang/rust:nightly-slim AS builder

RUN apt-get update && apt-get install -y \
  libssl-dev pkg-config sqlite3 && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  sqlite3 ca-certificates && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/segura-gateway .

VOLUME ["/data"]
ENV DATABASE_URL=sqlite://data/database.sqlite

EXPOSE 4206
CMD ["./segura-gateway"]
