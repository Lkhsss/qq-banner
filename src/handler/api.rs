use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    AppState,
    error::AppErr,
    handler::{AuthManager, UserStatusBack},
};
use qq_banner::model::{Manager, User};

use axum::{
    Form, Json,
    extract::{Path, State},
};

pub async fn ban(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.0;
    //验证密码
    let q = Manager::all()
        .filter(Manager::fields().name().eq(manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;
    if q.is_none() {
        return Err(AppErr::BadPassword);
    }
    //获取时间戳
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp_secs = since_the_epoch.as_secs();

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
    Ok(Json(UserStatusBack::banned(user)))
}

pub async fn unban(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.0;

    let q = Manager::all()
        .filter(Manager::fields().name().eq(manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;
    if q.is_none() {
        return Err(AppErr::BadPassword);
    }
    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    if let Some(u) = users {
        u.delete().exec(&mut db).await?;
    }
    println!("id: [{}]解除封禁", id);
    Ok(Json(UserStatusBack::unbanned(id)))
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
