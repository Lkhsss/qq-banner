use qq_banner::model::{Manager, Permission};

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
