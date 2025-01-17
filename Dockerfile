ARG APP_NAME=charizhard-ota

# Base image for building dependencies
FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

# Install necessary dependencies
RUN apk add --no-cache openssl-dev pkgconfig musl-dev perl make

# Add the Rust target for musl
RUN rustup target add x86_64-unknown-linux-musl

# Planner stage for dependency caching
FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage for compiling the application
FROM chef AS builder
ARG APP_NAME=charizhard-ota
ENV APP_NAME=${APP_NAME}
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN mv ./target/x86_64-unknown-linux-musl/release/${APP_NAME} /app/app

# Runtime stage with minimal base image
FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
RUN chmod +x /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]


