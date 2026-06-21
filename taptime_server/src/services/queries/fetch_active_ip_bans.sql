SELECT ip_cidr, reason
FROM ip_bans
WHERE revoked_at IS NULL
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY created_at DESC
