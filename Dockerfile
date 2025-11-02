FROM rust as builder 

WORKDIR /app
# Cache deps first
COPY Cargo.toml Cargo.lock ./
# Create a dummy src so "cargo build" can resolve deps without your full source (better caching)
RUN mkdir -p src
# Now bring in real source and rebuild
COPY src ./src
# Build for release
RUN cargo build --release

# ---- Runtime stage (minimal Ubuntu) ----
FROM ubuntu:24.04

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Keep appuser if you want, but DO NOT switch to it
RUN useradd -m appuser || true

COPY --from=builder /app/target/release/png2goo /usr/local/bin/png2goo
RUN chmod +x /usr/local/bin/png2goo

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /data
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["--help"]
