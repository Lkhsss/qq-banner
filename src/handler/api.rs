use super::*;

use crate::{
    AppState,
    database::Banner,
    error::AppErr,
    handler::{UserStatusBack, is_ban_expired, now_unix_secs, permission::get_permisson},
};
use qq_banner::model::{Manager, Permission, User};
use serde::Deserialize;

use axum::{
    Form, Json,
    extract::{Path, State},
};

// pub async fn ban(
//     Path(id): Path<u64>,
//     State(state): State<AppState>,
//     Form(form): Form<BanForm>,
// ) -> Result<Json<UserStatusBack>, AppErr> {
//     //获取时间
//     let timestamp_secs = now_unix_secs();
//     let mut db = state.db;
//     //验证密码
//     let requester = match Manager::all()
//         .filter(Manager::fields().name().eq(form.name.clone()))
//         .filter(Manager::fields().password().eq(form.password.clone()))
//         .first()
//         .exec(&mut db)
//         .await?
//     {
//         Some(r) => r,
//         None => return Err(AppErr::BadPassword),
//     };

//     //验证权限
//     let permisson = get_permisson(&mut db, &id.to_string()).await?;

//     //如果请求方权限小于被处理方权限，那么返回403
//     if requester.permission <= permisson {
//         return Err(AppErr::PermissonDenied);
//     }

//     let user = User {
//         id,
//         time: timestamp_secs,
//         duration: form.duration,
//         operator: requester.name,
//     };
//     // 封禁用户
//     let new_user = db.ban(user).await?;
//     Ok(Json(new_user))
// }

// #[derive(Debug, Deserialize)]
// pub struct BanForm {
//     pub name: String,
//     pub password: String,
//     #[serde(default)]
//     pub duration: u64,
// }

// pub async fn unban(
//     Path(id): Path<u64>,
//     State(state): State<AppState>,
//     Form(manager): Form<Manager>,
// ) -> Result<Json<UserStatusBack>, AppErr> {
//     let mut db = state.db;

//     let q = Manager::all()
//         .filter(Manager::fields().name().eq(manager.name))
//         .filter(Manager::fields().password().eq(manager.password))
//         .first()
//         .exec(&mut db)
//         .await?;
//     if q.is_none() {
//         return Err(AppErr::BadPassword);
//     }
//     let users = User::all()
//         .filter(User::fields().id().eq(id))
//         .first()
//         .exec(&mut db)
//         .await?;

//     if let Some(u) = users {
//         u.delete().exec(&mut db).await?;
//         //自增减1
//         METRIC_BANNED.fetch_sub(1, Ordering::Relaxed);
//     }
//     println!("id: [{}]解除封禁", id);
//     Ok(Json(UserStatusBack::unbanned(id)))
// }

pub async fn check(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.db;
    let now = now_unix_secs();
    let users = User::filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    match users {
        Some(u) => {
            if is_ban_expired(&u, now) {
                u.delete().exec(&mut db).await?;
                Ok(Json(UserStatusBack::unbanned(id)))
            } else {
                Ok(Json(UserStatusBack::banned(u)))
            }
        }
        None => Ok(Json(UserStatusBack::unbanned(id))),
    }
}

pub async fn add_manager(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.db;

    let m = Manager::filter(Manager::fields().name().eq(manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;

    //密码错误
    if m.is_none() {
        return Err(AppErr::BadPassword);
    }

    let exists = Manager::filter(Manager::fields().name().eq(&name))
        .first()
        .exec(&mut db)
        .await?;
    if exists.is_some() {
        return Err(AppErr::ManagerExists);
    }

    let password = Uuid::new_v4().simple().to_string();
    let manager = toasty::create!(Manager {
        name,
        password,
        permission: Permission::Admin as i16,
    })
    .exec(&mut db)
    .await?;

    Ok(Json(manager))
}
pub async fn del_manager(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<String, AppErr> {
    let mut db = state.db;

    let m = Manager::filter(Manager::fields().name().eq(manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;

    //密码错误
    if m.is_none() {
        return Err(AppErr::BadPassword);
    }

    // 删除
    Manager::filter_by_name(&name)
        .delete()
        .exec(&mut db)
        .await?;

    Ok(name)
}
