FROM rustlang/rust:nightly-slim AS builder
ARG BUSINESS_URL

RUN apt-get update && apt-get install -y \
  libssl-dev pkg-config sqlite3 && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN touch database.sqlite
RUN sqlite3 database.sqlite < init.sql
ENV DATABASE_URL=sqlite://database.sqlite
ENV BUSINESS_URL=${BUSINESS_URL}

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  sqlite3 ca-certificates && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/segura-gateway .
COPY --from=builder /usr/src/app/database.sqlite .

EXPOSE 3030

CMD ["./segura-gateway"]
