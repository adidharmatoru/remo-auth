# Stage 1: Build the Rust application
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/remo-auth

# Copy the actual source code
COPY . .

# Build the actual application
RUN cargo build --release

# Stage 2: Create a minimal image
FROM debian:bookworm-slim

# Install necessary libraries
RUN apt-get update 
# RUN apt-get install -y --no-install-recommends libssl1.1
RUN apt-get clean
RUN rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder
COPY --from=builder /usr/src/remo-auth/target/release/remo-auth /usr/local/bin/remo-auth

# Command to run the application
CMD ["remo-auth", "--address", "0.0.0.0:8444"]
