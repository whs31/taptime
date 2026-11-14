use crate::{
    clock::WorkClock, contracts::*, errors::SdkError, mapper::ProtoMapper,
    reports::ReportCalculator, timeline::TimelineCalculator,
};
use chrono_tz::Tz;
use std::sync::{Arc, Mutex};
use taptime_client::api::{
    DashboardRange, EventRequest, FlagRequest, ProfileUpdate, RegisterRequest, SettingsUpdate,
    WorkOverrideRequest,
};
use taptime_client::{Session, TapTimeClient};

#[derive(uniffi::Object)]
pub struct TapTimeSdk {
    client: TapTimeClient,
    session: Mutex<Option<Session>>,
}

impl TapTimeSdk {
    fn session(&self) -> Result<Session, SdkError> {
        let guard = self
            .session
            .lock()
            .map_err(|_| SdkError::input("Session lock poisoned"))?;
        let value = guard.as_ref().ok_or_else(SdkError::unauthorized)?;
        Session::new(value.token()).map_err(|_| SdkError::unauthorized())
    }
    fn save_session(&self, session: Session) -> Result<(), SdkError> {
        *self
            .session
            .lock()
            .map_err(|_| SdkError::input("Session lock poisoned"))? = Some(session);
        Ok(())
    }
    fn settings(input: SettingsInput) -> Result<SettingsUpdate, SdkError> {
        let time_zone: Tz = input
            .time_zone
            .parse()
            .map_err(|_| SdkError::input("Invalid IANA timezone"))?;
        let settings = input.settings;
        if settings.required_work_seconds < 0 || settings.lunch_seconds < 0 {
            return Err(SdkError::input("Negative work duration"));
        }
        Ok(SettingsUpdate {
            time_zone,
            required_work_hours: std::time::Duration::from_secs(
                settings.required_work_seconds as u64,
            ),
            lunch_break_duration: std::time::Duration::from_secs(settings.lunch_seconds as u64),
            weekends: settings
                .weekends
                .into_iter()
                .map(ProtoMapper::weekday)
                .collect(),
            remote_days: settings
                .remote_days
                .into_iter()
                .map(ProtoMapper::weekday)
                .collect(),
            start_date: settings
                .start_date
                .map(|d| ProtoMapper::date(&d))
                .transpose()?,
        })
    }
    fn event(input: EventInput) -> Result<EventRequest, SdkError> {
        if !(0..86400).contains(&input.local_seconds) {
            return Err(SdkError::input("Invalid wall time"));
        }
        let second = input.local_seconds as u32;
        let time = taptime_core::LocalTime::new(second / 3600, (second % 3600) / 60, second % 60)
            .map_err(|_| SdkError::input("Invalid wall time"))?;
        Ok(EventRequest {
            date: ProtoMapper::date(&input.date)?,
            time,
        })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl TapTimeSdk {
    #[uniffi::constructor]
    pub fn new(endpoint: String) -> Result<Arc<Self>, SdkError> {
        let client = TapTimeClient::new(&endpoint).map_err(|e| SdkError::api(e, false))?;
        Ok(Arc::new(Self {
            client,
            session: Mutex::new(None),
        }))
    }
    pub async fn login(&self, email: String, password: String) -> Result<AuthResult, SdkError> {
        if email.trim().is_empty() || password.is_empty() {
            return Err(SdkError::input("Email/password required"));
        }
        let response = self
            .client
            .login(&email, &password)
            .await
            .map_err(|e| SdkError::api(e, false))?;
        let token = response.session.token().to_string();
        let user = ProtoMapper::user(response.user)?;
        self.save_session(response.session)?;
        Ok(AuthResult { token, user })
    }
    pub async fn register(&self, input: RegisterInput) -> Result<AuthResult, SdkError> {
        let settings = Self::settings(SettingsInput {
            time_zone: input.time_zone,
            settings: input.settings,
        })?;
        let response = self
            .client
            .register(RegisterRequest {
                profile: ProfileUpdate {
                    name: input.name,
                    email: input.email,
                    organization: input.organization,
                },
                settings,
                password: input.password,
            })
            .await
            .map_err(|e| SdkError::api(e, true))?;
        let token = response.session.token().to_string();
        let user = ProtoMapper::user(response.user)?;
        self.save_session(response.session)?;
        Ok(AuthResult { token, user })
    }
    pub fn set_session_token(&self, token: String) -> Result<(), SdkError> {
        let session = Session::new(token).map_err(|_| SdkError::input("Invalid session token"))?;
        self.save_session(session)
    }
    pub fn clear_session_token(&self) {
        if let Ok(mut guard) = self.session.lock() {
            *guard = None;
        }
    }
    pub async fn fetch_user(&self) -> Result<UserDto, SdkError> {
        let user = self
            .client
            .get_user(&self.session()?)
            .await
            .map_err(|e| SdkError::api(e, false))?;
        ProtoMapper::user(user)
    }
    pub async fn fetch_day(&self, date: String) -> Result<DayDto, SdkError> {
        let date = ProtoMapper::date(&date)?;
        let day = self
            .client
            .get_day(&self.session()?, date)
            .await
            .map_err(|e| SdkError::api(e, false))?;
        ProtoMapper::day(day)
    }
    pub async fn fetch_dashboard(&self, query: DashboardQuery) -> Result<DashboardDto, SdkError> {
        let range = DashboardRange {
            range_start: ProtoMapper::date(&query.range_start)?,
            range_end: ProtoMapper::date(&query.range_end)?,
            month_start: ProtoMapper::date(&query.month_start)?,
            month_end: ProtoMapper::date(&query.month_end)?,
            today: ProtoMapper::date(&query.today)?,
        };
        let reply = self
            .client
            .get_dashboard_range(&self.session()?, range)
            .await
            .map_err(|e| SdkError::api(e, false))?;
        ProtoMapper::dashboard(reply)
    }
    pub async fn check_in(&self, input: EventInput) -> Result<(), SdkError> {
        let event = Self::event(input)?;
        self.client
            .add_check_in(&self.session()?, event)
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn check_out(&self, input: EventInput) -> Result<(), SdkError> {
        let event = Self::event(input)?;
        self.client
            .add_check_out(&self.session()?, event)
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn delete_event(&self, event_id: String) -> Result<(), SdkError> {
        let uuid =
            uuid::Uuid::parse_str(&event_id).map_err(|_| SdkError::input("Invalid event UUID"))?;
        self.client
            .delete_event(&self.session()?, uuid.into())
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn toggle_day_flag(&self, input: FlagInput) -> Result<(), SdkError> {
        let flag = match input.flag {
            DayFlag::Weekend => taptime_schema::DayFlag::Weekend,
            DayFlag::DayOff => taptime_schema::DayFlag::DayOff,
            DayFlag::Remote => taptime_schema::DayFlag::Remote,
            DayFlag::Vacation => taptime_schema::DayFlag::Vacation,
        };
        self.client
            .toggle_day_flag(
                &self.session()?,
                FlagRequest {
                    date: ProtoMapper::date(&input.date)?,
                    flag,
                },
            )
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn set_work_override(&self, input: WorkOverrideInput) -> Result<(), SdkError> {
        if input.required_work_seconds.is_some_and(|v| v < 0) {
            return Err(SdkError::input("Negative work duration"));
        }
        self.client
            .set_work_override(
                &self.session()?,
                WorkOverrideRequest {
                    date: ProtoMapper::date(&input.date)?,
                    required_work_hours: input
                        .required_work_seconds
                        .map(|v| std::time::Duration::from_secs(v as u64)),
                },
            )
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn update_profile(&self, input: ProfileInput) -> Result<UserDto, SdkError> {
        let reply = self
            .client
            .update_profile(
                &self.session()?,
                ProfileUpdate {
                    name: input.name,
                    email: input.email,
                    organization: input.organization,
                },
            )
            .await
            .map_err(|e| SdkError::api(e, true))?;
        ProtoMapper::user(reply)
    }
    pub async fn update_settings(&self, input: SettingsInput) -> Result<UserDto, SdkError> {
        let reply = self
            .client
            .update_settings(&self.session()?, Self::settings(input)?)
            .await
            .map_err(|e| SdkError::api(e, true))?;
        ProtoMapper::user(reply)
    }
    pub async fn update_rfid(&self, uid: Option<String>) -> Result<UserDto, SdkError> {
        let parsed = uid
            .map(|s| taptime_core::Uid::from_hex(&s).map_err(|_| SdkError::input("Invalid RFID")))
            .transpose()?;
        let reply = self
            .client
            .update_rfid(&self.session()?, parsed)
            .await
            .map_err(|e| SdkError::api(e, true))?;
        ProtoMapper::user(reply)
    }
    pub async fn delete_time_data(&self) -> Result<(), SdkError> {
        self.client
            .delete_time_data(&self.session()?)
            .await
            .map_err(|e| SdkError::api(e, true))
    }
    pub async fn delete_account(&self, password: String) -> Result<(), SdkError> {
        self.client
            .delete_account(&self.session()?, password)
            .await
            .map_err(|e| SdkError::api(e, true))?;
        self.clear_session_token();
        Ok(())
    }
    pub fn calculate_day(&self, input: DayCalculationInput) -> Result<DayMetricsDto, SdkError> {
        WorkClock::calculate(input)
    }
    pub fn build_timeline(&self, input: TimelineInput) -> Result<TimelineDto, SdkError> {
        TimelineCalculator::calculate(input)
    }
    pub fn build_report(&self, input: ReportInput) -> Result<ReportDto, SdkError> {
        ReportCalculator::calculate(input)
    }
}
