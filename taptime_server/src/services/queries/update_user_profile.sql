UPDATE users
SET name = $2,
    email = $3,
    organization = $4
WHERE id = $1
