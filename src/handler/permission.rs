use axum::Form;
use qq_banner::model::{Manager, Permission};
use toasty::Db;

use super::*;

pub async fn check_permisson(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    let mut db = state.db;

    let admin = Manager::filter(Manager::fields().name().eq(id.to_string()))
        .first()
        .exec(&mut db)
        .await?;
    match admin {
        Some(name) => {
            if name.permission < Permission::Admin.into() {
                return Err(AppErr::PermissonDenied);
            };
            return Ok(name.name);
        }
        None => return Err(AppErr::UserNotFound),
    }
}

/// # 获取数据库中权限
/// 如未在库中权限默认为-1
pub async fn get_permisson(db: &mut Db, id: &str) -> Result<i16, AppErr> {
    let user = Manager::filter(Manager::fields().name().eq(id))
        .first()
        .exec(db)
        .await?;
    match user {
        Some(u) => Ok(u.permission),
        None => Ok(-1),
    }
}
pub async fn handle_get_permisson(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    let permission = get_permisson(&mut db, &id.to_string()).await?;
    Ok(permission.to_string())
}

pub async fn get_password(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    let admin = Manager::filter(Manager::fields().name().eq("admin"))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;
    if admin.is_none() {
        return Err(AppErr::BadPassword);
    }

    let m = Manager::filter(Manager::fields().name().eq(id.to_string()))
        .first()
        .exec(&mut db)
        .await?;

    match m {
        Some(n) => Ok(n.password),
        None => Err(AppErr::UserNotFound),
    }
}
