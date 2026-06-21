# Production Deployment

TapTime ships a production Docker Compose stack for PostgreSQL, the Rust API server, and the SvelteKit web app.

## 1. Configure Environment

Copy the example file and edit the secrets:

```sh
cp .env.production.example .env.production
```

Set these values before starting the stack:

- `POSTGRES_PASSWORD`: use a long random password.
- `JWT_SECRET`: use a long random secret; changing it logs users out.
- `ADMIN_PASSWORD_HASH`: optional Argon2 hash for the admin CLI. Leave empty to disable admin login.
- `TRUST_PROXY_HEADERS`: set to `true` when the API is reachable only through your trusted reverse proxy.
- `PUBLIC_API_URL`: the API URL that the user's browser can reach.

Generate an admin password hash locally:

```sh
cargo run --manifest-path taptime_admin_cli/Cargo.toml -- hash-password
```

Argon2 hashes include a random salt, so the same password intentionally prints a
different PHC string each time. Any generated hash for the password will verify;
store one full string in `ADMIN_PASSWORD_HASH`.

Argon2 PHC strings contain `$` separators. In `.env.production`, wrap the hash
in single quotes so Docker Compose does not treat those fields as variable
interpolation:

```env
ADMIN_PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$...$...'
```

The default bind addresses expose both services only on host loopback:

```env
WEB_BIND=127.0.0.1
WEB_PORT=3000
API_BIND=127.0.0.1
API_PORT=50051
```

That shape is intended for a host-level reverse proxy such as Caddy, Nginx, or Traefik.

## 2. Start

From the repository root:

```sh
docker compose --env-file .env.production up -d --build
```

PostgreSQL data is stored in the named Docker volume `taptime_postgres_data`. The server runs database migrations on startup.

## 3. Reverse Proxy Notes

Terminate TLS in your host reverse proxy and forward traffic to:

- web: `http://127.0.0.1:3000`
- API: `http://127.0.0.1:50051`

Set `PUBLIC_API_URL` to the public API origin or routed API path, for example:

```env
PUBLIC_API_URL=https://api.example.com
```

Do not set `PUBLIC_API_URL` to `http://server:50051`; that name only exists inside Docker and is not reachable by browsers.

The web app uses gRPC-Web and can be proxied with normal HTTP proxying. The
admin CLI is a native tonic gRPC client, so the admin service path must be
proxied with HTTP/2 gRPC support if you run it through Nginx:

```nginx
server {
    listen 443 ssl;
    http2 on;
    server_name api.example.com;

    ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;

    # Native gRPC for taptime_admin_cli.
    location /com.whs31.taptime.services.AdminService/ {
        grpc_pass grpc://127.0.0.1:50051;
        grpc_set_header Host $host;
        grpc_set_header X-Real-IP $remote_addr;
        grpc_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        grpc_set_header X-Forwarded-Proto $scheme;
    }

    # gRPC-Web for the browser app.
    location / {
        proxy_pass http://127.0.0.1:50051;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Without that `grpc_pass` location, `taptime_admin_cli --admin-api-url=https://...`
can fail with native HTTP/2 errors such as `FRAME_SIZE_ERROR`.

## 4. Operations

View logs:

```sh
docker compose --env-file .env.production logs -f
```

Stop the stack:

```sh
docker compose --env-file .env.production down
```

Upgrade after pulling new code:

```sh
docker compose --env-file .env.production up -d --build
```

Run the admin TUI:

```sh
docker compose --env-file .env.production --profile admin run --rm admin_cli
```
