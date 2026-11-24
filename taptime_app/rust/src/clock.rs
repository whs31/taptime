use crate::{contracts::*, errors::SdkError, mapper::ProtoMapper};
use chrono::{TimeZone, Timelike, Utc};
use chrono_tz::Tz;

pub struct WorkClock;

impl WorkClock {
    pub fn calculate(input: DayCalculationInput) -> Result<DayMetricsDto, SdkError> {
        if input.work_target_seconds < 0
            || input.day.lunch_seconds < 0
            || input.work_target_seconds > 86400
            || input.day.lunch_seconds > 86400
        {
            return Err(SdkError::input("Invalid work duration"));
        }
        let date = ProtoMapper::date(&input.day.date)?;
        let zone: Tz = input
            .time_zone
            .parse()
            .map_err(|_| SdkError::input("Invalid IANA timezone"))?;
        let now = Utc
            .timestamp_opt(input.now_unix_seconds, 0)
            .single()
            .ok_or_else(|| SdkError::input("Invalid Unix time"))?;
        let live = now.with_timezone(&zone).date_naive() == date;
        let now_seconds = (now.with_timezone(&zone).time().num_seconds_from_midnight()) as i64;
        let mut sessions = Vec::new();
        let mut open: Option<i64> = None;
        let mut first: Option<i64> = None;
        let mut last: Option<i64> = None;
        let mut total = 0i64;
        for event in &input.day.events {
            if !(0..86400).contains(&event.local_seconds) {
                return Err(SdkError::input("Invalid event time"));
            }
            match event.kind {
                EventKind::CheckIn => {
                    open = Some(event.local_seconds);
                    first.get_or_insert(event.local_seconds);
                }
                EventKind::CheckOut => {
                    last = Some(event.local_seconds);
                    if let Some(start) = open.take() {
                        let worked = (event.local_seconds - start).max(0);
                        total = total.saturating_add(worked);
                        sessions.push(WorkSessionDto {
                            start_seconds: start,
                            end_seconds: Some(event.local_seconds),
                            worked_seconds: worked,
                        });
                    }
                }
            }
        }
        let core_day = taptime_core::Day {
            date,
            events: input
                .day
                .events
                .iter()
                .map(|event| {
                    let time = taptime_core::LocalTime::new(
                        (event.local_seconds / 3600) as u32,
                        ((event.local_seconds % 3600) / 60) as u32,
                        (event.local_seconds % 60) as u32,
                    )
                    .map_err(|_| SdkError::input("Invalid event time"))?;
                    Ok(match event.kind {
                        EventKind::CheckIn => taptime_core::Event::CheckIn(time),
                        EventKind::CheckOut => taptime_core::Event::CheckOut(time),
                    })
                })
                .collect::<Result<Vec<_>, SdkError>>()?,
            flags: taptime_core::DayFlags::empty(),
            required_work_hours: chrono::Duration::seconds(input.work_target_seconds),
            lunch_break_duration: chrono::Duration::seconds(input.day.lunch_seconds),
        };
        total = core_day.clocked_work_duration().num_seconds();
        let active = open.is_some() && live;
        if let Some(start) = open {
            let worked = if live {
                (now_seconds - start).max(0)
            } else {
                0
            };
            total = total.saturating_add(worked);
            sessions.push(WorkSessionDto {
                start_seconds: start,
                end_seconds: None,
                worked_seconds: worked,
            });
        }
        let presence = first.map(|start| {
            let end = if active {
                now_seconds
            } else {
                last.unwrap_or(start)
            };
            (end - start).max(0)
        });
        let target = input.work_target_seconds.max(0);
        let presence_target = if target == 0 {
            0
        } else {
            target.saturating_add(input.day.lunch_seconds.max(0))
        };
        let special = !input.day.flags.is_empty() || target == 0 || input.before_start_date;
        let balance = if special {
            0
        } else {
            let skipped =
                date < now.with_timezone(&zone).date_naive() && input.day.events.is_empty();
            presence
                .unwrap_or(0)
                .saturating_sub(if skipped { target } else { presence_target })
        };
        let expected = first.map(|start| start.saturating_add(presence_target));
        Ok(DayMetricsDto {
            date: input.day.date,
            is_working: active,
            worked_seconds: total,
            required_work_seconds: target,
            presence_seconds: presence,
            balance_seconds: balance,
            remaining_seconds: if special { 0 } else { (-balance).max(0) },
            first_check_in_seconds: first,
            last_check_out_seconds: if active { None } else { last },
            expected_end_seconds: expected,
            sessions,
        })
    }
}
