use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use jsonwebtoken::{DecodingKey, Validation, decode};
use qq_banner::{
    SALT,
    model::{Manager, User},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AppState, error::AppErr};

pub mod api;
pub mod webui;

#[derive(Debug, Deserialize)]
struct Claim {
    name: String,
    #[serde(rename = "exp")]
    _exp: i64,
}

pub struct AuthManager {
    pub name: String,
}

impl<S> FromRequestParts<S> for AuthManager
where
    S: Send + Sync,
    AppState: FromRef<S>,
    Key: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> = PrivateCookieJar::from_request_parts(parts, state)
            .await
            .expect("PrivateCookieJar extraction is infallible");

        let Some(token_cookie) = jar.get("token") else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "reason": "未登录：缺少token",
                })),
            )
                .into_response());
        };

        let token = token_cookie.value();
        let claims = match decode::<Claim>(
            token,
            &DecodingKey::from_secret(SALT.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => data.claims,
            Err(err) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "ok": false,
                        "reason": format!("token校验失败: {err}"),
                    })),
                )
                    .into_response());
            }
        };

        let app_state = AppState::from_ref(state);
        let mut db = app_state.0;

        match Manager::all()
            .filter(Manager::fields().name().eq(claims.name.clone()))
            .first()
            .exec(&mut db)
            .await
        {
            Ok(Some(_)) => Ok(Self { name: claims.name }),
            Ok(None) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "reason": "登录已失效：账号不存在",
                })),
            )
                .into_response()),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "ok": false,
                    "reason": format!("数据库错误: {err}"),
                })),
            )
                .into_response()),
        }
    }
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppErr> {
    let mut db = state.0;
    let users = User::all().exec(&mut db).await?;
    println!("{:?}", users);
    Ok(Json(users))
}

#[derive(Debug, Serialize)]
pub struct UserStatusBack {
    status: UserStatus,
    id: u64,
    time: u64,
}
#[derive(Debug, Serialize)]
pub enum UserStatus {
    Banned,
    Unbanned,
}

impl UserStatusBack {
    fn banned(u: User) -> Self {
        Self {
            status: UserStatus::Banned,
            id: u.id,
            time: u.time,
        }
    }
    fn unbanned(id: u64) -> Self {
        Self {
            status: UserStatus::Unbanned,
            id,
            time: 0,
        }
    }
}

pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
