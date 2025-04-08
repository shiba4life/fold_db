# Use the official Rust image as the base image
FROM rust:1.70-slim as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Use a smaller base image for the runtime
FROM debian:bullseye-slim

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/sample_social_app /app/sample_social_app

# Copy the static files
COPY static ./static

# Expose the port the app will run on
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["/app/sample_social_app"]
