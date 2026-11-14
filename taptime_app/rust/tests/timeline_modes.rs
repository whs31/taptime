use taptime_android_sdk::*;
#[test]
fn calendar_axis_covers_day() {
    let input = DayCalculationInput {
        day: DayDto {
            date: "2020-01-02".into(),
            events: vec![
                EventDto {
                    id: "".into(),
                    kind: EventKind::CheckIn,
                    local_seconds: 9 * 3600,
                },
                EventDto {
                    id: "".into(),
                    kind: EventKind::CheckOut,
                    local_seconds: 17 * 3600,
                },
            ],
            flags: vec![],
            required_work_seconds: 8 * 3600,
            lunch_seconds: 1800,
        },
        time_zone: "Europe/Moscow".into(),
        now_unix_seconds: 1_800_000_000,
        work_target_seconds: 8 * 3600,
        before_start_date: false,
    };
    let cal = taptime_android_sdk::timeline::TimelineCalculator::calculate(TimelineInput {
        calculation: input.clone(),
        mode: TimelineMode::CalendarDay,
    })
    .unwrap();
    let work = taptime_android_sdk::timeline::TimelineCalculator::calculate(TimelineInput {
        calculation: input,
        mode: TimelineMode::WorkDay,
    })
    .unwrap();
    assert_eq!((cal.axis_start_seconds, cal.axis_end_seconds), (0, 86400));
    assert_eq!(
        (work.axis_start_seconds, work.axis_end_seconds),
        (9 * 3600, 17 * 3600 + 1800)
    );
}

#[test]
fn work_axis_separates_break_and_overtime() {
    let input = DayCalculationInput {
        day: DayDto {
            date: "2026-11-14".into(),
            events: [
                (EventKind::CheckIn, 28800),
                (EventKind::CheckOut, 43200),
                (EventKind::CheckIn, 46800),
                (EventKind::CheckOut, 64800),
            ]
            .into_iter()
            .map(|(kind, local_seconds)| EventDto {
                id: String::new(),
                kind,
                local_seconds,
            })
            .collect(),
            flags: vec![],
            required_work_seconds: 28800,
            lunch_seconds: 1800,
        },
        time_zone: "UTC".into(),
        now_unix_seconds: 1800000000,
        work_target_seconds: 28800,
        before_start_date: false,
    };
    let result = taptime_android_sdk::timeline::TimelineCalculator::calculate(TimelineInput {
        calculation: input,
        mode: TimelineMode::WorkDay,
    })
    .unwrap();
    assert_eq!(result.axis_end_seconds, 59400);
    assert_eq!(result.overtime_seconds, 5400);
    assert!(
        result
            .segments
            .iter()
            .any(|segment| segment.kind == TimelineSegmentKind::Break
                && segment.start_seconds == 43200
                && segment.end_seconds == 46800)
    );
    assert!(
        result
            .segments
            .iter()
            .any(|segment| segment.kind == TimelineSegmentKind::Overtime
                && segment.start_seconds == 59400
                && segment.end_seconds == 64800)
    );
}
