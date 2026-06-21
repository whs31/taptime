DELETE FROM day_required_work_overrides
WHERE user_id = $1
  AND date_days = $2
