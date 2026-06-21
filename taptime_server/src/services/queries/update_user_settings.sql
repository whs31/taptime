UPDATE users
SET time_zone = $2,
    required_work_hours_secs = $3,
    lunch_break_duration_secs = $4,
    weekends = $5,
    remote_days = $6
WHERE id = $1
