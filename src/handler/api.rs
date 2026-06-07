use crate::{
    AppState,
    database::Banner,
    error::AppErr,
    handler::{UserStatusBack, now_unix_secs},
};
use qq_banner::model::User;

use axum::{
    Json,
    extract::{Path, State},
};

/// # 检查用户是否被封禁
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
            if toasty::Db::is_ban_expired(&u, now) {
                u.delete().exec(&mut db).await?;
                Ok(Json(UserStatusBack::unbanned(id)))
            } else {
                Ok(Json(UserStatusBack::banned(u)))
            }
        }
        None => Ok(Json(UserStatusBack::unbanned(id))),
    }
}
