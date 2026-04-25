use axum::{Json, extract::State};

use qq_banner::model::User;
use serde::{Deserialize, Serialize};

use crate::{AppState, error::AppErr};

pub mod api;
pub mod metrics;
pub mod webui;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    pub name: String,
    pub exp: i64,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppErr> {
    let mut db = state.db;
    let users = User::all().exec(&mut db).await?;

    Ok(Json(users))
}
pub async fn banned_user_count(State(state): State<AppState>) -> Result<String, AppErr> {
    let mut db = state.db;
    let users = User::all().exec(&mut db).await?;
    Ok(users.len().to_string())
}

#[derive(Debug, Serialize)]
pub struct UserStatusBack {
    status: UserStatus,
    id: u64,
    time: u64,
}
#[derive(Debug, Serialize)]
pub enum UserStatus {
    Banned,
    Unbanned,
}

impl UserStatusBack {
    fn banned(u: User) -> Self {
        Self {
            status: UserStatus::Banned,
            id: u.id,
            time: u.time,
        }
    }
    fn unbanned(id: u64) -> Self {
        Self {
            status: UserStatus::Unbanned,
            id,
            time: 0,
        }
    }
}

pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
