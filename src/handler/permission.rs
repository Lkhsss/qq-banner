use axum::Form;
use qq_banner::model::Manager;

use crate::database::Banner;
use tracing::{info, instrument};

use super::*;

// 供webui用
// pub async fn check_permisson(
//     Path(id): Path<u64>,
//     State(state): State<AppState>,
// ) -> Result<String, AppErr> {
//     let mut db = state.db;

//     let admin = Manager::filter(Manager::fields().name().eq(id.to_string()))
//         .first()
//         .exec(&mut db)
//         .await?;
//     match admin {
//         Some(name) => {
//             if name.permission < Permission::Admin.into() {
//                 return Err(AppErr::PermissonDenied);
//             };
//             Ok(name.name)
//         }
//         None => Err(AppErr::UserNotFound),
//     }
// }

#[instrument(name = "获取权限", skip(state))]
pub async fn handle_get_permisson(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    let mut db = state.db;
    let permission = db.get_permisson(&id.to_string()).await?;
    info!("查询用户 {id} 权限: {permission}");
    Ok(permission.to_string())
}

#[instrument(name = "获取密码", skip(state))]
pub async fn get_password(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Form(manager): Form<Manager>,
) -> Result<String, AppErr> {
    let mut db = state.db;

    let admin = Manager::filter(Manager::fields().name().eq("admin")) //FIXME
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
