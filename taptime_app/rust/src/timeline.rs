use crate::{clock::WorkClock, contracts::*, errors::SdkError, mapper::ProtoMapper};
use chrono::{TimeZone, Timelike, Utc};
use chrono_tz::Tz;

pub struct TimelineCalculator;

impl TimelineCalculator {
    pub fn calculate(input: TimelineInput) -> Result<TimelineDto, SdkError> {
        let metrics = WorkClock::calculate(input.calculation.clone())?;
        let zone: Tz = input
            .calculation
            .time_zone
            .parse()
            .map_err(|_| SdkError::input("Invalid IANA timezone"))?;
        let now = Utc
            .timestamp_opt(input.calculation.now_unix_seconds, 0)
            .single()
            .ok_or_else(|| SdkError::input("Invalid Unix timestamp"))?;
        let now_local = now.with_timezone(&zone);
        let today = ProtoMapper::date(&input.calculation.day.date)? == now_local.date_naive();
        let now_seconds = if today {
            Some(now_local.time().num_seconds_from_midnight() as i64)
        } else {
            None
        };
        let (axis_start, axis_end) = match input.mode {
            TimelineMode::CalendarDay => (0, 86400),
            TimelineMode::WorkDay => match metrics.first_check_in_seconds {
                Some(start) => (
                    start,
                    metrics
                        .expected_end_seconds
                        .unwrap_or(start + 1)
                        .max(start + 1),
                ),
                None => (0, 86400),
            },
        };
        let mut segments = Vec::new();
        let mut previous_end: Option<i64> = None;
        for session in &metrics.sessions {
            if let Some(end) = previous_end
                && session.start_seconds > end
            {
                segments.push(TimelineSegmentDto {
                    kind: TimelineSegmentKind::Break,
                    start_seconds: end,
                    end_seconds: session.start_seconds,
                });
            }
            let end = session
                .end_seconds
                .unwrap_or_else(|| {
                    if today {
                        now_seconds.unwrap_or(session.start_seconds)
                    } else {
                        session.start_seconds
                    }
                })
                .max(session.start_seconds);
            if end > session.start_seconds {
                let regular_end = end.min(axis_end);
                if regular_end > session.start_seconds {
                    segments.push(TimelineSegmentDto {
                        kind: TimelineSegmentKind::Work,
                        start_seconds: session.start_seconds,
                        end_seconds: regular_end,
                    });
                }
                if end > axis_end && input.mode == TimelineMode::WorkDay {
                    segments.push(TimelineSegmentDto {
                        kind: TimelineSegmentKind::Overtime,
                        start_seconds: axis_end.max(session.start_seconds),
                        end_seconds: end,
                    });
                }
            }
            previous_end = Some(end);
        }
        Ok(TimelineDto {
            mode: input.mode,
            axis_start_seconds: axis_start,
            axis_end_seconds: axis_end,
            now_seconds,
            overtime_seconds: if input.mode == TimelineMode::WorkDay {
                metrics
                    .presence_seconds
                    .unwrap_or(0)
                    .saturating_sub(axis_end - axis_start)
                    .max(0)
            } else {
                0
            },
            segments,
        })
    }
}
