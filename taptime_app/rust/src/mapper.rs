use crate::contracts::*;
use crate::errors::SdkError;
use chrono::NaiveDate;
use taptime_schema::{self as proto, event::EventType, services};

pub struct ProtoMapper;

impl ProtoMapper {
    pub fn date(value: &str) -> Result<NaiveDate, SdkError> {
        let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| SdkError::input("Invalid ISO date"))?;
        if date.format("%Y-%m-%d").to_string() != value {
            return Err(SdkError::input("Invalid ISO date"));
        }
        Ok(date)
    }
    pub fn duration(value: Option<prost_types::Duration>) -> Result<i64, SdkError> {
        let duration = value.ok_or_else(|| SdkError::input("Missing duration"))?;
        if duration.seconds < 0 || !(0..1_000_000_000).contains(&duration.nanos) {
            return Err(SdkError::input("Invalid duration"));
        }
        Ok(duration.seconds)
    }
    fn uid(value: &proto::Uuid) -> String {
        let bytes: uuid::Uuid = value.into();
        bytes.to_string()
    }
    fn date_string(value: Option<proto::Date>) -> Result<String, SdkError> {
        let date = value.ok_or_else(|| SdkError::input("Missing date"))?;
        let naive = NaiveDate::from_epoch_days(date.days_since_epoch)
            .ok_or_else(|| SdkError::input("Date out of range"))?;
        Ok(naive.format("%Y-%m-%d").to_string())
    }
    pub fn weekday(value: Weekday) -> proto::Weekday {
        match value {
            Weekday::Monday => proto::Weekday::Monday,
            Weekday::Tuesday => proto::Weekday::Tuesday,
            Weekday::Wednesday => proto::Weekday::Wednesday,
            Weekday::Thursday => proto::Weekday::Thursday,
            Weekday::Friday => proto::Weekday::Friday,
            Weekday::Saturday => proto::Weekday::Saturday,
            Weekday::Sunday => proto::Weekday::Sunday,
        }
    }
    fn from_weekday(value: i32) -> Result<Weekday, SdkError> {
        Ok(
            match proto::Weekday::try_from(value).map_err(|_| SdkError::input("Invalid weekday"))? {
                proto::Weekday::Monday => Weekday::Monday,
                proto::Weekday::Tuesday => Weekday::Tuesday,
                proto::Weekday::Wednesday => Weekday::Wednesday,
                proto::Weekday::Thursday => Weekday::Thursday,
                proto::Weekday::Friday => Weekday::Friday,
                proto::Weekday::Saturday => Weekday::Saturday,
                proto::Weekday::Sunday => Weekday::Sunday,
                _ => return Err(SdkError::input("Unspecified weekday")),
            },
        )
    }
    pub fn user(value: proto::User) -> Result<UserDto, SdkError> {
        let settings = value
            .settings
            .ok_or_else(|| SdkError::input("Missing user settings"))?;
        Ok(UserDto {
            id: value
                .id
                .as_ref()
                .map(Self::uid)
                .ok_or_else(|| SdkError::input("Missing user id"))?,
            name: value.name,
            email: value.email,
            organization: value.organization,
            time_zone: value
                .time_zone
                .map(|v| v.time_zone)
                .ok_or_else(|| SdkError::input("Missing timezone"))?,
            created_at: value
                .created_at
                .map(|v| v.seconds)
                .ok_or_else(|| SdkError::input("Missing creation timestamp"))?,
            last_seen: value.last_seen.map(|v| v.seconds),
            rfid_uid: value.rfid_uid.map(|v| {
                v.value
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<String>()
            }),
            settings: UserSettingsDto {
                required_work_seconds: Self::duration(settings.required_work_hours)?,
                lunch_seconds: Self::duration(settings.lunch_break_duration)?,
                weekends: settings
                    .weekends
                    .into_iter()
                    .map(Self::from_weekday)
                    .collect::<Result<Vec<_>, _>>()?,
                remote_days: settings
                    .remote_days
                    .into_iter()
                    .map(Self::from_weekday)
                    .collect::<Result<Vec<_>, _>>()?,
                start_date: settings
                    .start_date
                    .map(|d| Self::date_string(Some(d)))
                    .transpose()?,
            },
        })
    }
    pub fn day(value: proto::Day) -> Result<DayDto, SdkError> {
        let mut flags = Vec::new();
        for (bit, flag) in [
            (1, DayFlag::Weekend),
            (2, DayFlag::DayOff),
            (4, DayFlag::Remote),
            (8, DayFlag::Vacation),
        ] {
            if value.flags & bit != 0 {
                flags.push(flag);
            }
        }
        let mut events = Vec::new();
        for event in value.events {
            let (kind, t) = match event
                .event_type
                .ok_or_else(|| SdkError::input("Missing event type"))?
            {
                EventType::CheckIn(t) => (EventKind::CheckIn, t),
                EventType::CheckOut(t) => (EventKind::CheckOut, t),
            };
            if t.hour > 23 || t.minute > 59 || t.second > 59 {
                return Err(SdkError::input("Invalid event time"));
            }
            events.push(EventDto {
                id: event
                    .id
                    .as_ref()
                    .map(Self::uid)
                    .ok_or_else(|| SdkError::input("Missing event UUID"))?,
                kind,
                local_seconds: (t.hour * 3600 + t.minute * 60 + t.second) as i64,
            });
        }
        Ok(DayDto {
            date: Self::date_string(value.date)?,
            events,
            flags,
            required_work_seconds: Self::duration(value.required_work_hours)?,
            lunch_seconds: Self::duration(value.lunch_break_duration)?,
        })
    }
    pub fn summary(value: services::DaySummary) -> Result<DaySummaryDto, SdkError> {
        let balance = value
            .balance
            .ok_or_else(|| SdkError::input("Missing day balance"))?;
        let balance_seconds = match balance.balance_type {
            Some(proto::balance::BalanceType::Overtime(v)) => Self::duration(Some(v))?,
            Some(proto::balance::BalanceType::UnderTime(v)) => -Self::duration(Some(v))?,
            Some(proto::balance::BalanceType::Exact(_)) => 0,
            None => return Err(SdkError::input("Missing balance type")),
        };
        Ok(DaySummaryDto {
            day: Self::day(value.day.ok_or_else(|| SdkError::input("Missing day"))?)?,
            clocked_work_seconds: Self::duration(value.clocked_work)?,
            balance_seconds,
            skipped: value.skipped,
            full_day_worked: value.full_day_worked,
            required_work_hours_overridden: value.required_work_hours_overridden,
            work_target_seconds: Self::duration(value.work_target)?,
            before_start_date: value.before_start_date,
        })
    }
    pub fn dashboard(value: services::DashboardResponse) -> Result<DashboardDto, SdkError> {
        let stats = value
            .month_stats
            .ok_or_else(|| SdkError::input("Missing monthly stats"))?;
        Ok(DashboardDto {
            days: value
                .days
                .into_iter()
                .map(Self::summary)
                .collect::<Result<Vec<_>, _>>()?,
            month_stats: MonthStatsDto {
                total_clocked_work_seconds: Self::duration(stats.total_clocked_work)?,
                overtime_seconds: Self::duration(stats.overtime)?,
                undertime_seconds: Self::duration(stats.undertime)?,
                worked_days: stats.worked_days,
                remote_days: stats.remote_days,
                day_offs: stats.day_offs,
                vacation_days: stats.vacation_days,
                skipped_days: stats.skipped_days,
                full_weekend_work_days: stats.full_weekend_work_days,
                full_vacation_work_days: stats.full_vacation_work_days,
            },
        })
    }
}
