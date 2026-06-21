INSERT INTO day_required_work_overrides (user_id, date_days, required_work_hours_secs)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, date_days) DO UPDATE SET required_work_hours_secs = EXCLUDED.required_work_hours_secs
