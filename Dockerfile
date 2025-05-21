# Build stage
FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/dodo .

# Copy migrations
COPY --from=builder /usr/src/app/migrations ./migrations

# Copy .env file
COPY .env .

# Expose the port
EXPOSE 8080

# Run the application
CMD ["./dodo"] 