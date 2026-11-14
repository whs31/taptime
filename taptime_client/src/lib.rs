#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "api")]
pub mod auth;
#[cfg(feature = "api")]
pub use api::{ApiError, TapTimeClient, TodaySnapshot, decode_worked_time};
#[cfg(feature = "api")]
pub use auth::{LoginResult, Session};
#[cfg(feature = "backend")]
pub use auth::{clear_session, load_session, save_session, session_path};
#[cfg(feature = "backend")]
pub mod backend;
#[cfg(any(
  feature = "backend",
  all(unix, not(target_os = "android"), not(feature = "api"))
))]
pub mod ipc;
#[cfg(any(
  feature = "backend",
  all(unix, not(target_os = "android"), not(feature = "api"))
))]
pub mod storage;
