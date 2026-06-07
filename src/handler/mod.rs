use crate::{AppState, error::AppErr};
use axum::extract::Path;
use axum::extract::State;
use qq_banner::model::User;
use qq_banner::{METRIC_BANNED, METRIC_FAIL, METRIC_REQUEST, METRIC_SUCCESS, METRICS_DELAY};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

pub mod api;
pub mod banmanagement;
pub mod health;
pub mod info;
pub mod manager;
pub mod metrics;
pub mod permission;
pub mod webui;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    pub name: String,
    pub exp: i64,
}

#[derive(Debug, Serialize)]
pub struct UserStatusBack {
    status: UserStatus,
    id: u64,
    time: u64,
    duration: u64,
}
#[derive(Debug, Serialize)]
enum UserStatus {
    Banned,
    Unbanned,
}

impl UserStatusBack {
    pub fn banned<U: AsRef<User>>(user: U) -> Self {
        let u = user.as_ref();
        Self {
            status: UserStatus::Banned,
            id: u.id,
            time: u.time,
            duration: u.duration,
        }
    }
    pub fn unbanned(id: u64) -> Self {
        Self {
            status: UserStatus::Unbanned,
            id,
            time: 0,
            duration: 0,
        }
    }
}

pub(crate) fn now_unix_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
