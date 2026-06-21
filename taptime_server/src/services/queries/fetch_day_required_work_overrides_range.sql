SELECT date_days,
       required_work_hours_secs
FROM day_required_work_overrides
WHERE user_id = $1
  AND date_days BETWEEN $2 AND $3
ORDER BY date_days
