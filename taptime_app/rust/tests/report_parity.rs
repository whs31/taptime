use taptime_android_sdk::*;
fn summary(date: &str, balance: i64, seconds: i64) -> DaySummaryDto {
    DaySummaryDto {
        day: DayDto {
            date: date.into(),
            events: vec![],
            flags: vec![],
            required_work_seconds: 28800,
            lunch_seconds: 1800,
        },
        clocked_work_seconds: seconds,
        balance_seconds: balance,
        skipped: false,
        full_day_worked: false,
        required_work_hours_overridden: false,
        work_target_seconds: 28800,
        before_start_date: false,
    }
}
#[test]
fn reports_aggregate_server_summaries_once() {
    let sdk = TapTimeSdk::new("https://api.time.whs31.com".into()).unwrap();
    let report = sdk
        .build_report(ReportInput {
            range_start: "2026-11-01".into(),
            range_end: "2026-11-30".into(),
            days: vec![
                summary("2026-11-02", 3600, 28800),
                summary("2026-11-03", -1800, 25200),
            ],
        })
        .unwrap();
    assert_eq!(report.balance_seconds, 1800);
    assert_eq!(report.daily_balance.len(), 2);
    assert_eq!(report.daily_balance[1].cumulative_balance_seconds, 1800);
    assert_eq!(report.overtime_seconds, 3600);
    assert_eq!(report.undertime_seconds, 1800);
}
#[test]
fn duplicate_days_rejected() {
    let sdk = TapTimeSdk::new("https://api.time.whs31.com".into()).unwrap();
    assert!(
        sdk.build_report(ReportInput {
            range_start: "2026-11-01".into(),
            range_end: "2026-11-30".into(),
            days: vec![summary("2026-11-02", 0, 0), summary("2026-11-02", 0, 0)]
        })
        .is_err()
    );
}
