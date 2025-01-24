FROM rust:latest AS builder

# Set the target architecture
ARG TARGET= x86_64-unknown-linux-gnu
WORKDIR /app

COPY . .

RUN cargo build --release --target $TARGET

# Stage 2: Create a minimal image with the binary
FROM scratch

COPY --from=builder /app/target/$TARGET/release/prog /prog

# Set the command to execute the binary
CMD ["/prog"]
