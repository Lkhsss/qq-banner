use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AppState, error::AppErr};
use qq_banner::model::User;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

pub async fn ban(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp_secs = since_the_epoch.as_secs();

    let mut db = state.0;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;
    //存在则直接返回
    match users {
        Some(u) => {
            println!("id: [{}] already banned", id);
            return Ok(Json(UserStatusBack::banned(u)));
        }
        _ => (),
    }
    let user = toasty::create!(User {
        id,
        time: timestamp_secs,
    })
    .exec(&mut db)
    .await?;
    println!("Banned QQ : {}", user.id);
    Ok(Json(UserStatusBack::banned(user)))
}

pub async fn check(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.0;
    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    match users {
        Some(u) => return Ok(Json(UserStatusBack::banned(u))),
        None => return Ok(Json(UserStatusBack::unbanned(id))),
    }
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

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppErr> {
    let mut db = state.0;
    let users = User::all().exec(&mut db).await?;
    Ok(Json(users))
}
