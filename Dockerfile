ARG APP_NAME=charizhard-ota

FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

# Install necessary dependencies
RUN apk add --no-cache openssl-dev pkgconfig musl-dev
RUN rustup target add x86_64-unknown-linux-musl

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN mv ./target/x86_64-unknown-linux-musl/release/${APP_NAME} /app/app

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]

