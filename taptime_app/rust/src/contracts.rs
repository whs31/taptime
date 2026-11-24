use uniffi::{Enum, Record};

#[derive(Debug, Clone, Record)]
pub struct AuthResult {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Clone, Record)]
pub struct RegisterInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub organization: Option<String>,
    pub time_zone: String,
    pub settings: UserSettingsDto,
}

#[derive(Debug, Clone, Record)]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub organization: Option<String>,
    pub time_zone: String,
    pub created_at: i64,
    pub last_seen: Option<i64>,
    pub rfid_uid: Option<String>,
    pub settings: UserSettingsDto,
}

#[derive(Debug, Clone, Record)]
pub struct UserSettingsDto {
    pub required_work_seconds: i64,
    pub lunch_seconds: i64,
    pub weekends: Vec<Weekday>,
    pub remote_days: Vec<Weekday>,
    pub start_date: Option<String>,
}

#[derive(Debug, Clone, Record)]
pub struct DashboardQuery {
    pub range_start: String,
    pub range_end: String,
    pub month_start: String,
    pub month_end: String,
    pub today: String,
}

#[derive(Debug, Clone, Record)]
pub struct DashboardDto {
    pub days: Vec<DaySummaryDto>,
    pub month_stats: MonthStatsDto,
}

#[derive(Debug, Clone, Record)]
pub struct DayDto {
    pub date: String,
    pub events: Vec<EventDto>,
    pub flags: Vec<DayFlag>,
    pub required_work_seconds: i64,
    pub lunch_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct DaySummaryDto {
    pub day: DayDto,
    pub clocked_work_seconds: i64,
    pub balance_seconds: i64,
    pub skipped: bool,
    pub full_day_worked: bool,
    pub required_work_hours_overridden: bool,
    pub work_target_seconds: i64,
    pub before_start_date: bool,
}

#[derive(Debug, Clone, Record)]
pub struct EventDto {
    pub id: String,
    pub kind: EventKind,
    pub local_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct WorkSessionDto {
    pub start_seconds: i64,
    pub end_seconds: Option<i64>,
    pub worked_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct MonthStatsDto {
    pub total_clocked_work_seconds: i64,
    pub overtime_seconds: i64,
    pub undertime_seconds: i64,
    pub worked_days: u32,
    pub remote_days: u32,
    pub day_offs: u32,
    pub vacation_days: u32,
    pub skipped_days: u32,
    pub full_weekend_work_days: u32,
    pub full_vacation_work_days: u32,
}

#[derive(Debug, Clone, Record)]
pub struct DayMetricsDto {
    pub date: String,
    pub is_working: bool,
    pub worked_seconds: i64,
    pub required_work_seconds: i64,
    pub presence_seconds: Option<i64>,
    pub balance_seconds: i64,
    pub remaining_seconds: i64,
    pub first_check_in_seconds: Option<i64>,
    pub last_check_out_seconds: Option<i64>,
    pub expected_end_seconds: Option<i64>,
    pub sessions: Vec<WorkSessionDto>,
}

#[derive(Debug, Clone, Record)]
pub struct TimelineDto {
    pub mode: TimelineMode,
    pub axis_start_seconds: i64,
    pub axis_end_seconds: i64,
    pub now_seconds: Option<i64>,
    pub segments: Vec<TimelineSegmentDto>,
    pub overtime_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct TimelineSegmentDto {
    pub kind: TimelineSegmentKind,
    pub start_seconds: i64,
    pub end_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct ReportDto {
    pub range_start: String,
    pub range_end: String,
    pub worked_seconds: i64,
    pub planned_seconds: i64,
    pub balance_seconds: i64,
    pub overtime_seconds: i64,
    pub undertime_seconds: i64,
    pub months: Vec<MonthBucketDto>,
    pub daily_balance: Vec<DailyBalancePointDto>,
    pub exceptions: ExceptionDto,
    pub distribution: WorkDistributionDto,
}

#[derive(Debug, Clone, Record)]
pub struct MonthBucketDto {
    pub month_start: String,
    pub worked_seconds: i64,
    pub planned_seconds: i64,
    pub balance_seconds: i64,
    pub overtime_seconds: i64,
    pub undertime_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct DailyBalancePointDto {
    pub date: String,
    pub balance_seconds: i64,
    pub cumulative_balance_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct ExceptionDto {
    pub remote_days: u32,
    pub day_offs: u32,
    pub vacation_days: u32,
    pub skipped_days: u32,
    pub full_weekend_work_days: u32,
    pub full_vacation_work_days: u32,
}

#[derive(Debug, Clone, Record)]
pub struct WorkDistributionDto {
    pub working_days: u32,
    pub non_working_days: u32,
    pub worked_days: u32,
    pub overtime_days: u32,
    pub undertime_days: u32,
}

#[derive(Debug, Clone, Record)]
pub struct EventInput {
    pub date: String,
    pub local_seconds: i64,
}

#[derive(Debug, Clone, Record)]
pub struct FlagInput {
    pub date: String,
    pub flag: DayFlag,
}

#[derive(Debug, Clone, Record)]
pub struct WorkOverrideInput {
    pub date: String,
    pub required_work_seconds: Option<i64>,
}

#[derive(Debug, Clone, Record)]
pub struct ProfileInput {
    pub name: String,
    pub email: String,
    pub organization: Option<String>,
}

#[derive(Debug, Clone, Record)]
pub struct SettingsInput {
    pub time_zone: String,
    pub settings: UserSettingsDto,
}

#[derive(Debug, Clone, Record)]
pub struct DayCalculationInput {
    pub day: DayDto,
    pub time_zone: String,
    pub now_unix_seconds: i64,
    pub work_target_seconds: i64,
    pub before_start_date: bool,
}

#[derive(Debug, Clone, Record)]
pub struct TimelineInput {
    pub calculation: DayCalculationInput,
    pub mode: TimelineMode,
}

#[derive(Debug, Clone, Record)]
pub struct ReportInput {
    pub range_start: String,
    pub range_end: String,
    pub days: Vec<DaySummaryDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum EventKind {
    CheckIn,
    CheckOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DayFlag {
    Weekend,
    DayOff,
    Remote,
    Vacation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TimelineMode {
    WorkDay,
    CalendarDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TimelineSegmentKind {
    Work,
    Break,
    Overtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum SdkErrorCode {
    InvalidInput,
    Unauthenticated,
    PermissionDenied,
    NotFound,
    Conflict,
    Unavailable,
    Timeout,
    InvalidResponse,
    Internal,
}
