FROM rust:1.31 as build

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

RUN rustup default beta

WORKDIR /src

COPY . .

RUN cargo build

FROM ubuntu:latest

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=0 /src/target/debug/myscontroller-rs /usr/bin/

CMD ["/usr/bin/myscontroller-rs"]