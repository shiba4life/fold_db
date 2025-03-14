FROM rust:1.76-slim as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl build-essential git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release --bin datafold_node

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl-dev curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/datafold_node /app/datafold_node

# Create necessary directories
RUN mkdir -p /app/config /app/test_data

# Set environment variables
ENV NODE_CONFIG=/app/config/node_config.json
ENV NODE_PORT=3000

# Expose ports
# - HTTP API port
EXPOSE 3000
# - UDP discovery port
EXPOSE 9090/udp
# - TCP connection port
EXPOSE 9091

# Command to run
CMD ["/bin/sh", "-c", "/app/datafold_node --port $NODE_PORT"]
