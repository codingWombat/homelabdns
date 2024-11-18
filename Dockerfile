FROM rust:1.82 as build 
LABEL authors="codingwombat"

# create a new empty shell project
RUN USER=root cargo new --bin homelabdns
WORKDIR /homelabdns

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

# build for release;
RUN rm ./target/release/deps/homelabdns*
RUN cargo build --release

# our final base
FROM debian:buster-slim
COPY --from=build /homelabdns/target/release/homelabdns .

CMD ["./homelabdns"]