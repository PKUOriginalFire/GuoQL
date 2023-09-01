FROM rust:1.72-alpine as builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM alpine:latest
WORKDIR /guoql
COPY --from=builder /usr/local/cargo/bin/guoql /guoql/guoql
CMD ["./guoql", "./guoql.db", "--host", "0.0.0.0"]
