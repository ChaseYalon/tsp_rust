# -------------------------------
# Stage 1: Builder (Rust + Concorde)
# -------------------------------
FROM fedora:rawhide as builder

# Install development tools + required packages
RUN dnf -y --setopt=timeout=10 update \
 && dnf -y install gcc gcc-c++ make autoconf automake libtool pkgconf-pkg-config curl wget git gzip \
 && dnf clean all



# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory for Rust build
WORKDIR /rust-build

# Install nightly Rust toolchain and set as default
RUN rustup install nightly && rustup default nightly

# Copy the solver source
COPY ./solver ./solver

# Build the Rust project in release mode
WORKDIR /rust-build/solver
RUN cargo build --release

# Download Concorde TSP solver
WORKDIR /app/concorde
RUN wget -O concorde.gz https://www.math.uwaterloo.ca/tsp/concorde/downloads/codes/linux24/concorde.gz \
    && gunzip -f concorde.gz \
    && chmod +x concorde

# -------------------------------
# Stage 2: Runtime (Deno)
# -------------------------------
FROM denoland/deno:2.4.3

# Set working directory inside container
WORKDIR /app

# Copy backend/frontend and other resources
COPY ./backend ./backend
COPY ./frontend ./frontend
COPY ./certs ./certs
COPY ./backend/input ./backend/input
COPY ./backend/output ./backend/output
COPY ./solver ./solver

# Copy the Concorde executable from builder
COPY --from=builder /app/concorde ./concorde

# Copy compiled Rust binaries
COPY --from=builder /rust-build/solver/target ./solver/target

# Expose HTTPS port
EXPOSE 443

# Run the server with all permissions
CMD ["run", "-A", "backend/server.ts"]

