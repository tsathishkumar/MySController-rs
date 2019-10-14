FROM rust:1.38 as build

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /src

COPY . .

RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update && \
    apt-get -y install ca-certificates libudev-dev libssl-dev libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=0 /src/target/release/myscontroller-rs /usr/bin/

CMD ["/usr/bin/myscontroller-rs"]