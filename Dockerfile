# -------------------------------
# Stage 1: Builder (Rust)
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

#---------------------------------
# Stage 2: Build Concorde
#---------------------------------
# Download and prepare Concorde TSP solver
FROM fedora:38 AS concorde-builder
WORKDIR /app/concorde
RUN dnf -y --setopt=timeout=10 update \
    && dnf -y install gcc make tar curl autoconf automake wget

RUN wget https://www.math.uwaterloo.ca/tsp/concorde/downloads/codes/linux24/concorde.gz \
    && gunzip concorde.gz \
    && chmod +x concorde

#-------------------------------
# Stage 3: LKH
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
# Stage 4: Runtime (Deno + glibc + LKH + Concorde)
# -------------------------------
FROM fedora:rawhide
RUN dnf -y --setopt=timeout=10 update \
    && dnf -y install curl unzip tar gzip \
    && dnf clean all

# Install amd64 Deno explicitly
ENV DENO_VERSION=2.3.5
RUN curl -fsSL https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/deno-x86_64-unknown-linux-gnu.zip -o deno.zip \
    && unzip deno.zip -d /usr/local/bin \
    && rm deno.zip
ENV PATH="/usr/local/bin:${PATH}"

WORKDIR /app

# Copy backend, frontend, certs, and input/output
COPY ./backend ./backend
COPY ./frontend ./frontend
RUN cp -r ./certs ./certs || true
COPY ./backend/input ./backend/input
COPY ./backend/output ./backend/output
COPY ./solver ./solver

# Copy Concorde executable from builder
COPY --from=concorde-builder /app/concorde ./concorde

# Copy compiled Rust binaries from builder
COPY --from=builder /rust-build/solver/target ./solver/target

# Copy LKH binary from lkh-builder
COPY --from=lkh-builder /app/lkh ./app/lkh
RUN chmod +x ./app/lkh/lkh

# Expose HTTPS port
EXPOSE 80

# Run Deno server
CMD ["deno", "run", "-A", "backend/server.ts"]
