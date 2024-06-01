# Use the official Rust image as a builder
FROM rust:1.78 as builder

# Set the working directory
WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal image for the final stage
FROM debian:latest

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/pms_v_0 /usr/local/bin/pms_v_0

# Set the startup command
CMD ["pms_v_0"]