use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::database::Banner;
use crate::error::AppErr;
use crate::extracter::{AdminOrAbove, AuthManager};
use crate::handler::permission::get_permisson;
use crate::handler::{UserStatusBack, now_unix_secs};

use axum::Json;
use axum::extract::{Path, Query, State};
use qq_banner::model::User;


#[derive(Debug, Deserialize)]
pub struct BanQuery {
    pub duration: Option<u64>,
}

pub async fn ban(
    Path(id): Path<u64>,
    Query(params): Query<BanQuery>,
    State(state): State<AppState>,
    operator: AuthManager<AdminOrAbove>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let timestamp_secs = now_unix_secs();
    let mut db = state.db;
    //检查操作人权限
    let permisson = get_permisson(&mut db, &id.to_string()).await?;

    if operator.permission <= permisson.into() {
        return Err(AppErr::PermissonDenied);
    }

    let new_user = User {
        id,
        time: timestamp_secs,
        duration: params.duration.unwrap_or(0),
        operator: operator.name,
    };
    let banner = db.ban(new_user).await?;

    println!("Banned QQ : {}", banner.id);
    Ok(Json(banner))
}
