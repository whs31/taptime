SELECT date_days,
       flags
FROM day_flags
WHERE user_id = $1
  AND date_days BETWEEN $2 AND $3
ORDER BY date_days
