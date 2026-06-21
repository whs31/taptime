SELECT date_days,
       id,
       event_kind,
       hour,
       minute,
       second
FROM events
WHERE user_id = $1
  AND date_days BETWEEN $2 AND $3
ORDER BY date_days, hour, minute, second
