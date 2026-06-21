SELECT user_id,
       password_hash
FROM user_credentials
WHERE user_id = $1
