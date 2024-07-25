FROM rust:alpine as build

WORKDIR /app

RUN apk add --update alpine-sdk

RUN mkdir -p /app

COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

RUN cargo build --release

FROM rust:alpine as prod

COPY --from=build /app/target/release/spam_blocker /bin/spam_blocker

ENTRYPOINT [ "spam_blocker" ]