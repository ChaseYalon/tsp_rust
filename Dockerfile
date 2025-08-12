# Multi-stage build: First stage for Rust compilation
FROM rust:latest as rust-builder

# Set working directory for Rust build
WORKDIR /rust-build

# Install the latest nightly toolchain and set it as default
RUN rustup install nightly && rustup default nightly

# Copy the solver directory
COPY ./solver ./solver

# Build the Rust project in release mode
WORKDIR /rust-build/solver
RUN cargo build --release

# Second stage: Deno runtime with compiled Rust binary
FROM denoland/deno:2.4.3

# Set working directory inside container
WORKDIR /app

# Copy your backend source code into the container
COPY ./backend ./backend
COPY ./frontend ./frontend
COPY ./certs ./certs
COPY ./input ./input
COPY ./output ./output

# Copy the entire solver directory from the host (includes source)
COPY ./solver ./solver

# Copy the compiled Rust binary from the builder stage to replace the one in solver
COPY --from=rust-builder /rust-build/solver/target ./solver/target

# Expose HTTPS port
EXPOSE 443

# Run the server with all permissions
CMD ["run", "-A", "backend/server.ts"]
