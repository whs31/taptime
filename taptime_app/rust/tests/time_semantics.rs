use taptime_android_sdk::*;
#[test]
fn open_session_does_not_extend_historical_day() {
    let input = DayCalculationInput {
        day: DayDto {
            date: "2020-01-02".into(),
            events: vec![EventDto {
                id: "".into(),
                kind: EventKind::CheckIn,
                local_seconds: 9 * 3600,
            }],
            flags: vec![],
            required_work_seconds: 8 * 3600,
            lunch_seconds: 1800,
        },
        time_zone: "Europe/Moscow".into(),
        now_unix_seconds: 1_800_000_000,
        work_target_seconds: 8 * 3600,
        before_start_date: false,
    };
    let result = taptime_android_sdk::clock::WorkClock::calculate(input).unwrap();
    assert_eq!(result.worked_seconds, 0);
    assert!(!result.is_working);
}

fn calculation(
    date: &str,
    seconds: &[(EventKind, i64)],
    now: i64,
    zone: &str,
) -> DayCalculationInput {
    DayCalculationInput {
        day: DayDto {
            date: date.into(),
            events: seconds
                .iter()
                .map(|(kind, local_seconds)| EventDto {
                    id: String::new(),
                    kind: *kind,
                    local_seconds: *local_seconds,
                })
                .collect(),
            flags: vec![],
            required_work_seconds: 28800,
            lunch_seconds: 1800,
        },
        time_zone: zone.into(),
        now_unix_seconds: now,
        work_target_seconds: 28800,
        before_start_date: false,
    }
}

#[test]
fn multiple_sessions_follow_core_presence_balance() {
    let input = calculation(
        "2026-11-14",
        &[
            (EventKind::CheckIn, 28800),
            (EventKind::CheckOut, 43200),
            (EventKind::CheckIn, 46800),
            (EventKind::CheckOut, 59400),
        ],
        1800000000,
        "UTC",
    );
    let result = taptime_android_sdk::clock::WorkClock::calculate(input).unwrap();
    assert_eq!(result.worked_seconds, 27000);
    assert_eq!(result.presence_seconds, Some(30600));
    assert_eq!(result.balance_seconds, 0);
    assert_eq!(result.sessions.len(), 2);
}

#[test]
fn skipped_day_excludes_lunch_from_undertime() {
    let result = taptime_android_sdk::clock::WorkClock::calculate(calculation(
        "2020-01-02",
        &[],
        1800000000,
        "UTC",
    ))
    .unwrap();
    assert_eq!(result.balance_seconds, -28800);
    assert_eq!(result.first_check_in_seconds, None);
}

#[test]
fn dst_closed_sessions_preserve_server_wall_clock_semantics() {
    for date in ["2026-03-29", "2026-10-25"] {
        let result = taptime_android_sdk::clock::WorkClock::calculate(calculation(
            date,
            &[(EventKind::CheckIn, 5400), (EventKind::CheckOut, 12600)],
            1800000000,
            "Europe/Berlin",
        ))
        .unwrap();
        assert_eq!(result.worked_seconds, 7200);
    }
}

#[test]
fn account_timezone_controls_day_rollover() {
    use chrono::TimeZone;
    let now = chrono::Utc
        .with_ymd_and_hms(2026, 11, 14, 22, 0, 0)
        .unwrap()
        .timestamp();
    let input = calculation(
        "2026-11-14",
        &[(EventKind::CheckIn, 72000)],
        now,
        "Europe/Moscow",
    );
    assert!(
        !taptime_android_sdk::clock::WorkClock::calculate(input)
            .unwrap()
            .is_working
    );
    let input = calculation("2026-11-14", &[(EventKind::CheckIn, 72000)], now, "UTC");
    assert_eq!(
        taptime_android_sdk::clock::WorkClock::calculate(input)
            .unwrap()
            .worked_seconds,
        7200
    );
}
