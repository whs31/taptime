FROM rust:1.88-bookworm AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends libprotobuf-dev protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*

COPY taptime_schema/Cargo.toml taptime_schema/Cargo.lock ./taptime_schema/
COPY taptime_schema/build.rs ./taptime_schema/build.rs
COPY taptime_schema/schema ./taptime_schema/schema
COPY taptime_schema/src ./taptime_schema/src
COPY taptime_admin_cli/Cargo.toml taptime_admin_cli/Cargo.lock ./taptime_admin_cli/
COPY taptime_admin_cli/src ./taptime_admin_cli/src

WORKDIR /app/taptime_admin_cli
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/taptime_admin_cli/target/release/taptime_admin_cli /usr/local/bin/taptime_admin_cli

ENTRYPOINT ["taptime_admin_cli"]
