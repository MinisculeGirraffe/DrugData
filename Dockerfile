FROM rust:latest as builder
WORKDIR /usr/src/drug-data
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/drug-data/target \ 
    cargo build --release

RUN cargo install --path .


# Runtime image
FROM debian:bullseye-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/drug_data /app/drug_data