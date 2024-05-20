FROM rust:latest
WORKDIR /usr/src/app

COPY Cargo.toml .env entrypoint.sh ./

RUN mkdir src
COPY src/main.rs src/lib.rs src/

RUN mkdir migrations
COPY migrations/0001_create_table.sql migrations/

RUN mkdir tracks

RUN ./entrypoint.sh
