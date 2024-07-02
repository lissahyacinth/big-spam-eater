FROM rust:buster as build

ENV TZ=Europe/London

WORKDIR /app

RUN apt update -y && \
    apt install -y \
    curl \
    build-essential \
    libssl1.0 && \
    apt-get clean &&\
    rm -rf /var/lib/apt/lists/* 

RUN mkdir -p /app

COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

RUN cargo build --release

ENTRYPOINT [ "cargo" ]
CMD ["run", "--release"]