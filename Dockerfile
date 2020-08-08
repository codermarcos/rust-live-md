FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo install

CMD ["live-md"]