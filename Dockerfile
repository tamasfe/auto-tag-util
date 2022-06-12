FROM rust:1.61 as builder

RUN apt-get update && apt-get install -y zlib1g-dev libc-dev libssl-dev

WORKDIR /build

ADD src src
ADD Cargo.toml .
ADD Cargo.lock .

RUN cargo build --release

FROM bitnami/minideb:latest

RUN apt-get update && apt-get install -y git openssh-client

WORKDIR /auto-tag

COPY --from=builder /build/target/release/auto-tag /usr/bin/auto-tag

ENTRYPOINT [ "/usr/bin/auto-tag" ]
