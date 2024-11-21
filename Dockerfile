FROM rust:bookworm as build 
LABEL authors="codingwombat"
LABEL org.opencontainers.image.source = "https://github.com/codingWombat/homelabdns"

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
FROM debian:bookworm
COPY --from=build /homelabdns/target/release/homelabdns .

CMD ["./homelabdns"]