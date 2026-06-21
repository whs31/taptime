CREATE TABLE IF NOT EXISTS user_bans (
  id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  reason     TEXT        NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS user_bans_user_active
  ON user_bans (user_id)
  WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS ip_bans (
  id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  ip_cidr    TEXT        NOT NULL,
  reason     TEXT        NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS ip_bans_active
  ON ip_bans (revoked_at, expires_at);

CREATE TABLE IF NOT EXISTS user_ip_addresses (
  user_id       UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  ip            TEXT        NOT NULL,
  first_seen    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  request_count BIGINT      NOT NULL DEFAULT 1,
  PRIMARY KEY (user_id, ip)
);
