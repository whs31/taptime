CREATE TABLE IF NOT EXISTS day_required_work_overrides (
  user_id                  UUID   NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  date_days                INT    NOT NULL,
  required_work_hours_secs BIGINT NOT NULL CHECK (required_work_hours_secs > 0),
  PRIMARY KEY (user_id, date_days)
);
