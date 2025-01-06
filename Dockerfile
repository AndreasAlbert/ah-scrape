FROM rust:1.83-alpine3.21 AS build

RUN apk add musl-dev
#RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rust.sh && sh /tmp/rust.sh -y

COPY . /app/
WORKDIR /app/
RUN cargo build --release

FROM alpine:3.21.0 AS app
COPY --from=build /app/target/release/ah-scrape /app/release