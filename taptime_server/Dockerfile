FROM rust:1.88-bookworm AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends libprotobuf-dev protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*

COPY taptime_core/Cargo.toml taptime_core/Cargo.lock ./taptime_core/
COPY taptime_schema/Cargo.toml taptime_schema/Cargo.lock ./taptime_schema/
COPY taptime_schema/build.rs ./taptime_schema/build.rs
COPY taptime_schema/schema ./taptime_schema/schema
COPY taptime_server/Cargo.toml taptime_server/Cargo.lock ./taptime_server/

COPY taptime_core/src ./taptime_core/src
COPY taptime_schema/src ./taptime_schema/src
COPY taptime_server/src ./taptime_server/src
COPY taptime_server/migrations ./taptime_server/migrations

WORKDIR /app/taptime_server
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --create-home --home-dir /var/lib/taptime --shell /usr/sbin/nologin taptime \
  && mkdir -p /var/lib/taptime/.local/share/TapTime/logs \
  && chown -R taptime:taptime /var/lib/taptime

COPY --from=builder /app/taptime_server/target/release/taptime_server /usr/local/bin/taptime_server

USER taptime
ENV HOME=/var/lib/taptime

EXPOSE 50051

ENTRYPOINT ["taptime_server"]
CMD ["--address", "0.0.0.0:50051"]
