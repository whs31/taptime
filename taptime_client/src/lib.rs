#[cfg(feature = "backend")]
pub mod api;
#[cfg(feature = "backend")]
pub mod auth;
#[cfg(feature = "backend")]
pub mod tracker;

#[cfg(feature = "backend")]
pub use api::{ApiError, TapTimeClient, TodaySnapshot, decode_worked_time};
#[cfg(feature = "backend")]
pub use auth::{LoginResult, Session, clear_session, load_session, save_session, session_path};
#[cfg(feature = "backend")]
pub use tracker::WorkTracker;
#[cfg(feature = "backend")]
pub mod backend;
pub mod ipc;
pub mod storage;
