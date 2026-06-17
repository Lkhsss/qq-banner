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
use tracing::{info, instrument};

/// # 检查用户是否被封禁
#[instrument(skip(state), name = "检查封禁状态")]
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
                info!("用户 {id} 封禁已过期，已解封");
                Ok(Json(UserStatusBack::unbanned(id)))
            } else {
                info!("用户 {id} 仍在封禁中");
                Ok(Json(UserStatusBack::banned(u)))
            }
        }

        None => {
            info!("用户 {id} 未被封禁");
            Ok(Json(UserStatusBack::unbanned(id)))
        }
    }
}
