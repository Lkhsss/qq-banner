use crate::database::banned_user_count;
use crate::{AppState, error::AppErr};
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use qq_banner::model::User;
use qq_banner::{
    METRIC_BANNED, METRIC_FAIL, METRIC_REQUEST, METRIC_SUCCESS, METRICS_DELAY, PAGING_DEFAULT,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use uuid::Uuid;

pub mod api;
pub mod health;
pub mod metrics;
pub mod permission;
pub mod webui;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    pub name: String,
    pub exp: i64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Paging {
    page: Option<usize>,
    size: Option<usize>,
    order: Option<Order>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Paging {
    fn default() -> Self {
        Self {
            page: Some(1),
            size: Some(PAGING_DEFAULT),
            order: Some(Order::Asc),
        }
    }
}
pub async fn list(
    State(state): State<AppState>,
    Query(paging): Query<Paging>,
) -> Result<Json<Vec<User>>, AppErr> {
    let mut db = state.db;
    let page = paging.page.unwrap_or(1);
    let size = paging.size.unwrap_or(PAGING_DEFAULT);
    let order = paging.order.unwrap_or(Order::Desc);

    let offset = (page - 1) * size;
    // 排序
    let ord = match order {
        Order::Asc => User::fields().time().asc(),
        Order::Desc => User::fields().time().desc(),
    };

    let users = User::all()
        .order_by(ord)
        .limit(size)
        .offset(offset)
        .exec(&mut db)
        .await?;

    let now = now_unix_secs();
    let mut active_users = Vec::with_capacity(users.len());

    for user in users {
        if is_ban_expired(&user, now) {
            user.delete().exec(&mut db).await?;
        } else {
            active_users.push(user);
        }
    }

    Ok(Json(active_users))
}
pub async fn banned_user_count_handle(State(state): State<AppState>) -> Result<String, AppErr> {
    let mut db = state.db;
    let count = banned_user_count(&mut db).await?;

    Ok(count.to_string())
}

#[derive(Debug, Serialize)]
pub struct UserStatusBack {
    status: UserStatus,
    id: u64,
    time: u64,
    duration: u64,
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
            duration: u.duration,
        }
    }
    fn unbanned(id: u64) -> Self {
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

pub(crate) fn is_ban_expired(user: &User, now: u64) -> bool {
    user.duration != 0 && now >= user.time.saturating_add(user.duration)
}

pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
