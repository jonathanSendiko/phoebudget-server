# STAGE 1: Build
FROM rust:latest as builder

WORKDIR /app

# --- FIX 1: Install System Dependencies for OpenSSL ---
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the source code
COPY . .

# --- FIX 2: Copy the .sqlx folder for offline mode ---
COPY .sqlx .sqlx

# Touch main.rs to force rebuild
RUN touch src/main.rs

# --- FIX 3: Tell SQLx to use the offline file, not the real DB ---
ENV SQLX_OFFLINE=true

RUN cargo build --release

# STAGE 2: Run
FROM debian:bookworm-slim

# Install Runtime Dependencies (OpenSSL again, but for running)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/phoebudget .

# COPY .env .

EXPOSE 3000

CMD ["./phoebudget"]