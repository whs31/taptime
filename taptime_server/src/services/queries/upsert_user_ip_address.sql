INSERT INTO user_ip_addresses (user_id, ip, first_seen, last_seen, request_count)
VALUES ($1, $2, NOW(), NOW(), 1)
ON CONFLICT (user_id, ip)
DO UPDATE SET
  last_seen = NOW(),
  request_count = user_ip_addresses.request_count + 1
