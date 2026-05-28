use axum::{
    Json,
    extract::{Path, State},
};
use qq_banner::{
    ADMIN_USER,
    model::{Manager, Permission},
};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppErr,
    extracter::{AuthManager, SuperAdminOnly},
};

/// # 增加管理员
pub async fn add_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    _: AuthManager<SuperAdminOnly>,
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

    Ok(Json(manager))
}

/// # 减少管理员
pub async fn del_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    _: AuthManager<SuperAdminOnly>,
) -> Result<String, AppErr> {
    let mut db = state.db;

    if id == ADMIN_USER {
        return Err(AppErr::PermissonDenied);
    }

    // 删除
    Manager::filter_by_name(&id).delete().exec(&mut db).await?;

    Ok(id)
}

pub async fn refresh_password_manager(
    Path(id): Path<String>,
    State(state): State<AppState>,
    _auth: AuthManager<SuperAdminOnly>,
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
        Some(m) => Ok(Json(m)),
        None => Err(AppErr::Database_Unhealth),
    }
}

pub async fn list_manager(
    State(state): State<AppState>,
    _: AuthManager<SuperAdminOnly>,
) -> Result<Json<Vec<Manager>>, AppErr> {
    let mut db = state.db;
    let users = Manager::all().exec(&mut db).await?;
    Ok(Json(users))
}
