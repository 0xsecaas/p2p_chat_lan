# Use Ubuntu 24.04 as the base image
FROM ubuntu:24.04

# Set working directory
WORKDIR /app

# Copy the pre-built Rust binary
COPY target/release/p2p-walkie-talkie /app/p2p-walkie-talkie

# Expose the broadcasting port
EXPOSE 9999

# Set the entrypoint
ENTRYPOINT ["/app/p2p-walkie-talkie"]
