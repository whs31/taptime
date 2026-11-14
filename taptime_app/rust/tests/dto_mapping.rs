use taptime_android_sdk::*;
#[test]
fn event_and_date_contract_keeps_seconds() {
    let input = EventInput {
        date: "2026-11-14".into(),
        local_seconds: 86399,
    };
    assert_eq!(input.local_seconds, 23 * 3600 + 59 * 60 + 59);
    assert_eq!(input.date.len(), 10);
}
#[test]
fn optional_settings_preserve_explicit_zero() {
    let override_input = WorkOverrideInput {
        date: "2026-11-14".into(),
        required_work_seconds: Some(0),
    };
    assert_eq!(override_input.required_work_seconds, Some(0));
}

#[test]
fn malformed_proto_fields_are_rejected() {
    use taptime_android_sdk::mapper::ProtoMapper;
    assert!(
        ProtoMapper::duration(Some(prost_types::Duration {
            seconds: -1,
            nanos: 0
        }))
        .is_err()
    );
    assert!(ProtoMapper::date("2026-1-2").is_err());
    let day = taptime_schema::Day {
        date: Some(taptime_schema::Date {
            days_since_epoch: i32::MAX,
        }),
        required_work_hours: Some(prost_types::Duration {
            seconds: 28800,
            nanos: 0,
        }),
        lunch_break_duration: Some(prost_types::Duration {
            seconds: 1800,
            nanos: 0,
        }),
        ..Default::default()
    };
    assert!(ProtoMapper::day(day).is_err());
}

#[test]
fn mapping_keeps_event_uuid_and_local_seconds() {
    let day = taptime_schema::Day {
        date: Some(
            chrono::NaiveDate::from_ymd_opt(2026, 11, 14)
                .unwrap()
                .into(),
        ),
        events: vec![taptime_schema::Event {
            id: Some(taptime_schema::Uuid {
                most_significant_bits: 0,
                least_significant_bits: 1,
            }),
            event_type: Some(taptime_schema::event::EventType::CheckIn(
                taptime_schema::LocalTime {
                    hour: 8,
                    minute: 9,
                    second: 10,
                },
            )),
        }],
        required_work_hours: Some(prost_types::Duration {
            seconds: 28800,
            nanos: 0,
        }),
        lunch_break_duration: Some(prost_types::Duration {
            seconds: 1800,
            nanos: 0,
        }),
        ..Default::default()
    };
    let dto = taptime_android_sdk::mapper::ProtoMapper::day(day).unwrap();
    assert_eq!(dto.events[0].id, "00000000-0000-0000-0000-000000000001");
    assert_eq!(dto.events[0].local_seconds, 29350);
}
