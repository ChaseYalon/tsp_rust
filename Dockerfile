# -------------------------------
# Stage 1: Builder (Rust + Concorde)
# -------------------------------
FROM fedora:rawhide AS builder

# Install development tools + required packages
RUN dnf -y --setopt=timeout=10 update \
 && dnf -y install gcc gcc-c++ make autoconf automake libtool pkgconf-pkg-config curl wget git gzip \
 && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /rust-build

# Install nightly Rust toolchain and set as default
RUN rustup install nightly && rustup default nightly

# Copy solver source and build
COPY ./solver ./solver
WORKDIR /rust-build/solver
RUN cargo build --release

# Download and prepare Concorde TSP solver
WORKDIR /app/concorde
RUN wget -O concorde.gz https://www.math.uwaterloo.ca/tsp/concorde/downloads/codes/linux24/concorde.gz \
    && gunzip -f concorde.gz \
    && chmod +x concorde

#-------------------------------
# Stage 2: LKH
#-------------------------------
FROM fedora:rawhide AS lkh-builder
WORKDIR /app/lkh
RUN dnf -y --setopt=timeout=10 update \
    && dnf -y install wget tar make gcc
RUN wget http://webhotel4.ruc.dk/~keld/research/LKH/LKH-2.0.11.tgz \
    && tar xzvf LKH-2.0.11.tgz \
    && cd LKH-2.0.11 \
    && make \
    && cd .. \
    && mv LKH-2.0.11/LKH lkh

# ------------------------------- 
# Stage 3: Runtime (Deno + glibc + LKH + Concorde)
# -------------------------------
FROM fedora:rawhide
RUN dnf -y --setopt=timeout=10 update \
    && dnf -y install curl unzip \
    && dnf clean all

# Install Deno
RUN curl -fsSL https://deno.land/install.sh | sh
ENV PATH="/root/.deno/bin:${PATH}"

WORKDIR /app

# Copy backend, frontend, certs, and input/output
COPY ./backend ./backend
COPY ./frontend ./frontend
COPY ./certs ./certs
COPY ./backend/input ./backend/input
COPY ./backend/output ./backend/output
COPY ./solver ./solver

# Copy Concorde executable from builder
COPY --from=builder /app/concorde ./concorde

# Copy compiled Rust binaries from builder
COPY --from=builder /rust-build/solver/target ./solver/target

# Copy LKH binary from lkh-builder
COPY --from=lkh-builder /app/lkh ./app/lkh

# Make LKH executable
RUN chmod +x ./app/lkh/lkh

# Expose HTTPS port
EXPOSE 443

# Run Deno server
CMD ["deno", "run", "-A", "backend/server.ts"]
