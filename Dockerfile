FROM rust:1.67-slim as builder

WORKDIR /usr/src/datafold
COPY . .

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/datafold/target/release/datafold_node /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chmod 777 /data

# Create socket directory
RUN mkdir -p /var/run && chmod 777 /var/run

# Expose the API port
EXPOSE 8080

# Create a startup script
RUN echo '#!/bin/bash\n\
datafold_node &\n\
PID=$!\n\
sleep 5\n\
echo "Datafold API ready"\n\
wait $PID' > /usr/local/bin/start.sh && \
chmod +x /usr/local/bin/start.sh

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/start.sh"]
