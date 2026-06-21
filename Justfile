alias gen := generate
alias fmt := format

set shell := ["bash", "-cu"]

api_url := env("PUBLIC_API_URL", "http://127.0.0.1:50051")

default:
    @echo "Usage: just <command> [options]"
    @just --list

[working-directory("taptime_schema")]
generate-ts:
    @cargo test --features=serde,grpc,client,server,typescript

[working-directory("taptime_proto")]
install-proto-if-needed:
    @if [ ! -d node_modules ] || [ ! -f node_modules/.package-lock.json ] || [ package.json -nt node_modules/.package-lock.json ] || [ package-lock.json -nt node_modules/.package-lock.json ]; then \
        npm install; \
    else \
        echo "taptime_proto npm dependencies are up to date"; \
    fi

[working-directory("taptime_proto")]
generate-proto: install-proto-if-needed
    @npm run generate

[working-directory("taptime_proto")]
generate-proto-if-needed: install-proto-if-needed
    @marker="src/taptime/services/auth_connect.ts"; \
    stamp="node_modules/.cache/taptime_proto/generated.stamp"; \
    newer_schema=""; \
    if [ -f "$stamp" ]; then \
        newer_schema="$(find ../taptime_schema -path ../taptime_schema/target -prune -o \( -name '*.proto' -o -name 'buf.yaml' \) -newer "$stamp" -print -quit)"; \
    fi; \
    if [ ! -f "$marker" ] || [ ! -f "$stamp" ] || [ -n "$newer_schema" ] || [ buf.gen.yaml -nt "$stamp" ] || [ package.json -nt "$stamp" ] || [ package-lock.json -nt "$stamp" ]; then \
        npm run generate; \
        mkdir -p "$(dirname "$stamp")"; \
        touch "$stamp"; \
    else \
        echo "protobuf schema is up to date"; \
    fi

generate: generate-ts generate-proto

dev: generate-proto-if-needed
    @set -euo pipefail; \
    server_pid=""; \
    web_pid=""; \
    cleanup() { \
        if [ -n "${server_pid:-}" ]; then kill -- "-$server_pid" 2>/dev/null || true; fi; \
        if [ -n "${web_pid:-}" ]; then kill -- "-$web_pid" 2>/dev/null || true; fi; \
    }; \
    trap cleanup INT TERM EXIT; \
    setsid just server dev & \
    server_pid="$!"; \
    setsid env PUBLIC_API_URL="{{api_url}}" just web dev & \
    web_pid="$!"; \
    wait -n "$server_pid" "$web_pid"; \
    status="$?"; \
    cleanup; \
    trap - EXIT; \
    exit "$status"

[working-directory("taptime_schema")]
test-schema:
    @cargo test --features=grpc,client,server

[working-directory("taptime_core")]
test-core:
    @cargo test

test: test-schema test-core

[working-directory("taptime_core")]
format-core:
    @cargo +nightly fmt

[working-directory("taptime_schema")]
format-schema:
    @cargo +nightly fmt

format: format-core format-schema
    @just server format

mod web 'taptime_web'
mod server 'taptime_server'
