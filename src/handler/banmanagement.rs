use std::sync::atomic::Ordering;

use qq_banner::{METRIC_BANNED, PAGING_DEFAULT};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::database::Banner;
use crate::error::AppErr;
use crate::extracter::{AdminOrAbove, AuthManager};
use crate::handler::{UserStatusBack, now_unix_secs};

use axum::Json;
use axum::extract::{Path, Query, State};
use qq_banner::model::{Permission, User};

use tracing::{info, instrument};

#[derive(Debug, Deserialize)]
pub struct BanQuery {
    pub duration: Option<u64>,
    pub operator: Option<String>,
}

#[instrument(name = "封禁", skip(state, operator, params))]
pub async fn ban(
    Path(id): Path<u64>,
    Query(params): Query<BanQuery>,
    State(state): State<AppState>,
    operator: AuthManager<AdminOrAbove>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let timestamp_secs = now_unix_secs();
    let mut db = state.db;

    if id.to_string() == operator.name {
        return Err(AppErr::SelfOperationProhibited);
    }

    // 增加operater的指定
    let permisson = db.get_permisson(&id.to_string()).await?;

    if operator.permission <= permisson.into() {
        return Err(AppErr::PermissonDenied);
    }

    let duration = match params.duration {
        Some(d) => d.to_string(),
        None => String::from("永久"),
    };
    // 如果是超级管理员权限，允许指定操作人
    let operator_select = if operator.permission == Permission::SuperAdmin {
        match params.operator {
            Some(p) => {
                info!(
                    "超级管理员[{}]封禁[{}](指定操作人[{}]) 时长:{}",
                    &operator.name, id, &p, duration
                );
                p
            }
            None => {
                info!(
                    "超级管理员[{}]封禁[{}] 时长:{}",
                    &operator.name, id, duration
                );
                operator.name
            }
        }
    } else {
        info!("[{}]封禁[{}] 时长:{}", &operator.name, id, duration);
        operator.name
    };

    let new_user = User {
        id,
        time: timestamp_secs,
        duration: params.duration.unwrap_or(0),
        operator: operator_select,
    };

    let banner = db.ban(new_user).await?;

    Ok(Json(banner))
}

#[instrument(name = "取消封禁", skip(state))]
pub async fn unban(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    operator: AuthManager<AdminOrAbove>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.db;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    if let Some(u) = users {
        u.delete().exec(&mut db).await?;
        METRIC_BANNED.fetch_sub(1, Ordering::Relaxed);
        info!("管理员[{}]解封了用户 {id}", operator.name);
    } else {
        info!(
            "管理员[{}]尝试解封用户 {id}，但该用户未被封禁",
            operator.name
        );
    }
    Ok(Json(UserStatusBack::unbanned(id)))
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Filter {
    pub filter: Option<String>,
}

#[instrument(name = "封禁列表", skip(state))]
pub async fn list(
    State(state): State<AppState>,
    Query(paging): Query<Paging>,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<User>>, AppErr> {
    let mut db = state.db;
    let page = paging.page.unwrap_or(1);
    let size = paging.size.unwrap_or(PAGING_DEFAULT);
    let order = paging.order.unwrap_or(Order::Desc);

    let offset = (page - 1) * size;
    let ord = match order {
        Order::Asc => User::fields().time().asc(),
        Order::Desc => User::fields().time().desc(),
    };

    let mut users = User::all()
        .order_by(ord)
        .limit(size)
        .offset(offset)
        .exec(&mut db)
        .await?;

    let now = now_unix_secs();
    let mut active_users = Vec::with_capacity(users.len());

    if let Some(f) = filter.filter {
        users.retain(|x| x.id.to_string().contains(&f))
    }
    for user in users {
        if toasty::Db::is_ban_expired(&user, now) {
            user.delete().exec(&mut db).await?;
        } else {
            active_users.push(user);
        }
    }
    Ok(Json(active_users))
}

#[instrument(name = "封禁统计", skip(state))]
pub async fn banned_user_count_handle(
    State(state): State<AppState>,
    Query(filiter): Query<Filter>,
) -> Result<String, AppErr> {
    match filiter.filter {
        Some(f) => {
            let mut db = state.db;
            let users = User::all().exec(&mut db).await?;
            let now = now_unix_secs();
            let mut count = 0;

            for user in users {
                if toasty::Db::is_ban_expired(&user, now) {
                    user.delete().exec(&mut db).await?;
                } else {
                    if user.id.to_string().contains(&f) {
                        count += 1;
                    }
                }
            }
            Ok(count.to_string())
        }
        None => Ok(METRIC_BANNED.load(Ordering::Relaxed).to_string()),
    }
}

#[instrument(name = "举报", skip(state))]
pub async fn report(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    operator: AuthManager<AdminOrAbove>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    let count = db.report(id).await?;
    info!("管理员[{}]举报了用户 {id}，累计 {count} 次", operator.name);
    Ok(count.to_string())
}
#[instrument(name = "查看举报", skip(state))]
pub async fn get_report(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    Ok(db.get_report(id).await?.to_string())
}

#[instrument(name = "清空举报", skip(state))]
pub async fn clean_report(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    operator: AuthManager<AdminOrAbove>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    let count = db.clean_report(id).await?;
    info!(
        "管理员[{}]清空了用户 {id} 的举报记录，之前共 {count} 次",
        operator.name
    );
    Ok(count.to_string())
}
