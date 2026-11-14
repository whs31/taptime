use crate::{contracts::*, errors::SdkError, mapper::ProtoMapper};
use chrono::Datelike;
use std::collections::{BTreeMap, HashSet};

pub struct ReportCalculator;

impl ReportCalculator {
    pub fn calculate(input: ReportInput) -> Result<ReportDto, SdkError> {
        let start = ProtoMapper::date(&input.range_start)?;
        let end = ProtoMapper::date(&input.range_end)?;
        if start > end {
            return Err(SdkError::input("Invalid report range"));
        }
        let mut seen = HashSet::new();
        let mut months: BTreeMap<(i32, u32), MonthBucketDto> = BTreeMap::new();
        let mut points = Vec::new();
        let mut exceptions = ExceptionDto {
            remote_days: 0,
            day_offs: 0,
            vacation_days: 0,
            skipped_days: 0,
            full_weekend_work_days: 0,
            full_vacation_work_days: 0,
        };
        let mut distribution = WorkDistributionDto {
            working_days: 0,
            non_working_days: 0,
            worked_days: 0,
            overtime_days: 0,
            undertime_days: 0,
        };
        let mut totals = (0i64, 0i64, 0i64, 0i64, 0i64);
        let mut days = input.days;
        days.sort_by(|a, b| a.day.date.cmp(&b.day.date));
        for summary in days {
            let date = ProtoMapper::date(&summary.day.date)?;
            if date < start || date > end || !seen.insert(date) {
                return Err(SdkError::input("Duplicate or out-of-range report day"));
            }
            if summary.before_start_date {
                continue;
            }
            let excluded = summary.day.flags.contains(&DayFlag::Weekend)
                || summary.day.flags.contains(&DayFlag::DayOff)
                || summary.day.flags.contains(&DayFlag::Vacation);
            if excluded {
                distribution.non_working_days += 1;
            } else {
                distribution.working_days += 1;
            }
            if summary.clocked_work_seconds > 0 {
                distribution.worked_days += 1;
            }
            if summary.balance_seconds > 0 {
                distribution.overtime_days += 1;
            }
            if summary.balance_seconds < 0 {
                distribution.undertime_days += 1;
            }
            if summary.skipped {
                exceptions.skipped_days += 1;
            }
            if summary.day.flags.contains(&DayFlag::Remote) {
                exceptions.remote_days += 1;
            }
            if summary.day.flags.contains(&DayFlag::DayOff) {
                exceptions.day_offs += 1;
            }
            if summary.day.flags.contains(&DayFlag::Vacation) {
                exceptions.vacation_days += 1;
            }
            if summary.day.flags.contains(&DayFlag::Weekend) && summary.full_day_worked {
                exceptions.full_weekend_work_days += 1;
            }
            if summary.day.flags.contains(&DayFlag::Vacation) && summary.full_day_worked {
                exceptions.full_vacation_work_days += 1;
            }
            let worked = summary.clocked_work_seconds;
            let planned = if excluded {
                0
            } else {
                summary.work_target_seconds
            };
            let balance = summary.balance_seconds;
            totals.0 += worked;
            totals.1 += planned;
            totals.2 += balance;
            totals.3 += balance.max(0);
            totals.4 += (-balance).max(0);
            points.push(DailyBalancePointDto {
                date: summary.day.date.clone(),
                balance_seconds: balance,
                cumulative_balance_seconds: totals.2,
            });
            let month = months
                .entry((date.year(), date.month()))
                .or_insert_with(|| MonthBucketDto {
                    month_start: format!("{:04}-{:02}-01", date.year(), date.month()),
                    worked_seconds: 0,
                    planned_seconds: 0,
                    balance_seconds: 0,
                    overtime_seconds: 0,
                    undertime_seconds: 0,
                });
            month.worked_seconds += worked;
            month.planned_seconds += planned;
            month.balance_seconds += balance;
            month.overtime_seconds += balance.max(0);
            month.undertime_seconds += (-balance).max(0);
        }
        Ok(ReportDto {
            range_start: input.range_start,
            range_end: input.range_end,
            worked_seconds: totals.0,
            planned_seconds: totals.1,
            balance_seconds: totals.2,
            overtime_seconds: totals.3,
            undertime_seconds: totals.4,
            months: months.into_values().collect(),
            daily_balance: points,
            exceptions,
            distribution,
        })
    }
}
