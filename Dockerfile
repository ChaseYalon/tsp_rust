# Multi-stage build: First stage for Rust compilation
FROM debian:bookworm as builder

RUN apt update
RUN apt update && apt install -y curl build-essential wget git
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory for Rust build
WORKDIR /rust-build

# Install the latest nightly toolchain and set it as default
RUN rustup install nightly && rustup default nightly

# Copy the solver directory
COPY ./solver ./solver

# Build the Rust project in release mode
WORKDIR /rust-build/solver
RUN cargo build --release

# Clone concorde

WORKDIR /app/concorde
#Get concorde executable
RUN wget -O concorde.gz https://www.math.uwaterloo.ca/tsp/concorde/downloads/codes/linux24/concorde.gz \
    && gunzip -f concorde.gz \
    && chmod +x concorde


FROM denoland/deno:2.4.3

# Set working directory inside container
WORKDIR /app

# Copy your backend source code into the container
COPY ./backend ./backend
COPY ./frontend ./frontend
COPY ./certs ./certs
COPY ./backend/input ./backend/input
COPY ./backend/output ./backend/output
COPY  --from=builder ./app/concorde ./concorde
COPY ./solver ./solver

# Copy the compiled Rust binary from the builder stage to replace the one in solver
COPY --from=builder /rust-build/solver/target ./solver/target

# Expose HTTPS port
EXPOSE 443

# Run the server with all permissions
CMD ["run", "-A", "backend/server.ts"]
