SELECT reason
FROM user_bans
WHERE user_id = $1
  AND revoked_at IS NULL
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY created_at DESC
LIMIT 1
