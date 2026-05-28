use std::sync::atomic::Ordering;

use qq_banner::{METRIC_BANNED, PAGING_DEFAULT};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::database::Banner;
use crate::error::AppErr;
use crate::extracter::{AdminOrAbove, AuthManager};
use crate::handler::{UserStatusBack, is_ban_expired, now_unix_secs};

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
    let permisson = db.get_permisson(&id.to_string()).await?;

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

/// TODO
/// 和ban一样业务分离
pub async fn unban(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    _: AuthManager<AdminOrAbove>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.db;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    if let Some(u) = users {
        u.delete().exec(&mut db).await?;
        //自增减1
        METRIC_BANNED.fetch_sub(1, Ordering::Relaxed);
    }
    println!("webui: id: [{}]解除封禁", id);
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
    // 排序
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

    //筛选数据
    match filter.filter {
        Some(f) => users.retain(|x| x.id.to_string().contains(&f)),
        None => (),
    }
    for user in users {
        if is_ban_expired(&user, now) {
            user.delete().exec(&mut db).await?;
        } else {
            active_users.push(user);
        }
    }
    Ok(Json(active_users))
}

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
                if is_ban_expired(&user, now) {
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
