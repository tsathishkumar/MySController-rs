FROM rust:1.38 as build

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

RUN USER=root cargo new --bin myscontroller-rs
WORKDIR /myscontroller-rs


COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release

RUN rm src/*.rs

COPY . .
RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=0 /myscontroller-rs/target/release/myscontroller-rs /usr/bin/

CMD ["/usr/bin/myscontroller-rs"]