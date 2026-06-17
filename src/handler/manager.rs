use axum::{
    Json,
    extract::{Path, State},
};
use qq_banner::{
    ADMIN_USER,
    model::{Manager, Permission},
};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppErr,
    extracter::{AuthManager, SuperAdminOnly},
};

/// # 增加管理员
#[instrument(name = "添加管理员", skip(state))]
pub async fn add_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    operator: AuthManager<SuperAdminOnly>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.db;

    let exists = Manager::filter(Manager::fields().name().eq(&id))
        .first()
        .exec(&mut db)
        .await?;
    if exists.is_some() {
        return Err(AppErr::ManagerExists);
    }

    let password = Uuid::new_v4().simple().to_string();
    let manager = toasty::create!(Manager {
        name: id,
        password,
        permission: Permission::Admin as i16,
    })
    .exec(&mut db)
    .await?;

    info!(
        "超级管理员[{}]添加了管理员[{}]",
        operator.name, manager.name
    );
    Ok(Json(manager))
}

/// # 减少管理员
#[instrument(name = "删除管理员", skip(state))]
pub async fn del_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    operator: AuthManager<SuperAdminOnly>,
) -> Result<String, AppErr> {
    let mut db = state.db;

    if id == ADMIN_USER {
        return Err(AppErr::SelfOperationProhibited);
    }
    // 删除
    Manager::filter_by_name(&id).delete().exec(&mut db).await?;

    info!("超级管理员[{}]删除了管理员[{}]", operator.name, id);
    Ok(id)
}

#[instrument(name = "刷新管理员密码", skip(state))]
pub async fn refresh_password_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    operator: AuthManager<SuperAdminOnly>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.db;
    let password = Uuid::new_v4().simple().to_string();

    Manager::filter(Manager::fields().name().eq(&id))
        .update()
        .password(password)
        .exec(&mut db)
        .await?;
    let manager = Manager::filter(Manager::fields().name().eq(&id))
        .first()
        .exec(&mut db)
        .await?;

    match manager {
        Some(m) => {
            info!("超级管理员[{}]刷新了管理员[{}]的密码", operator.name, id);
            Ok(Json(m))
        }
        None => Err(AppErr::DatabaseUnhealth),
    }
}

#[instrument(name = "管理员列表", skip(state))]
pub async fn list_manager(
    State(state): State<AppState>,
    _: AuthManager<SuperAdminOnly>,
) -> Result<Json<Vec<Manager>>, AppErr> {
    let mut db = state.db;
    let users = Manager::all().exec(&mut db).await?;
    Ok(Json(users))
}
