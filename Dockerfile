# Compile application
FROM rust:latest as builder
WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/app/target \
	--mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/usr/local/cargo/git \
	--mount=type=cache,target=/usr/local/rustup \
    cargo build --release -C opt-level=3 -C lto \
    objcopy --compress-debug-sections target/release/drug_data ./drug_data

RUN cargo install --path .

FROM alpine:latest as tailscale
WORKDIR /app
COPY . ./
ENV TSFILE=tailscale_1.26.1_amd64.tgz
RUN wget https://pkgs.tailscale.com/stable/${TSFILE} && tar xzf ${TSFILE} --strip-components=1


# Runtime image
FROM debian:bullseye-slim

# Run as "app" user
RUN useradd -ms /bin/bash app
RUN set -eux; \
	export DEBIAN_FRONTEND=noninteractive; \
	apt update; \
	apt install --yes --no-install-recommends bind9-dnsutils iputils-ping iproute2 ca-certificates iptables; \
	apt clean autoclean; \
	apt autoremove --yes; \
	rm -rf /var/lib/{apt,dpkg,cache,log}/; \
	echo "Installed base utils!"RUN mkdir -p /var/run/tailscale /var/cache/tailscale /var/lib/tailscale

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory

COPY --from=tailscale /app/tailscaled /app/tailscaled
COPY --from=tailscale /app/tailscale /app/tailscale

COPY --from=builder /usr/src/drug-data/target/release/drug_data /app/drug_data
