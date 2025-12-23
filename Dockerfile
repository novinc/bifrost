# Building Stage
ARG RUST_VERSION=1.92
FROM rust:${RUST_VERSION}-slim-trixie AS build
WORKDIR /app
COPY LICENSE LICENSE
ENV PATH="/.cargo/bin:$PATH"

RUN apt-get -y update && apt-get install -y --no-install-recommends pkg-config libssl-dev

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY doc ./doc

# Create a dummy src to allow dependency compilation caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --locked --release
RUN rm -rf src

COPY src ./src

# Build the actual application
RUN cargo build --locked --release

# Final Stage
FROM debian:trixie-slim AS final
RUN apt-get -y update && apt-get install -y --no-install-recommends libssl3 ca-certificates
WORKDIR /app
COPY --from=build /app/target/release/bifrost /app/bifrost

CMD ["/app/bifrost"]
