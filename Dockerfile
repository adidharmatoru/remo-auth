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

# Environment variables for ICE servers configuration:
# STUN_SERVERS: Comma-separated list of STUN server URLs (e.g., "stun:stun.example.com:3478,stun:stun2.example.com:3478")
#
# Option 1: TURN servers with shared credentials
# TURN_SERVERS: Comma-separated list of TURN server URLs (e.g., "turn:turn.example.com:3478")
# TURN_USERNAME: Username for TURN server authentication
# TURN_CREDENTIAL: Password for TURN server authentication
#
# Option 2: TURN servers with individual credentials (preferred)
# TURN_SERVER_CONFIGS: Comma-separated list of TURN server configurations with individual credentials
#                      Format: "url|username|credential,url2|username2|credential2"
#                      Example: "turn:turn1.example.com:3478|user1|pass1,turn:turn2.example.com:3478|user2|pass2"
#
# ICE_SERVER_WHITELIST: Comma-separated list of whitelisted peer IDs that can access ICE servers

# Command to run the application
CMD ["remo-auth", "--address", "0.0.0.0:8444"]
