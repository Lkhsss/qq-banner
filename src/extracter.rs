use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use jsonwebtoken::{DecodingKey, Validation, decode};
use qq_banner::SALT;
use qq_banner::model::{Manager, Permission};
use serde_json::json;
use std::marker::PhantomData;

use crate::{AppState, handler::Claim};

pub trait PermissionPolicy {
    fn allows(permission: Permission) -> bool;
}

pub struct AnyPermission;

impl PermissionPolicy for AnyPermission {
    fn allows(_: Permission) -> bool {
        true
    }
}

pub struct AdminOrAbove;

impl PermissionPolicy for AdminOrAbove {
    fn allows(permission: Permission) -> bool {
        matches!(permission, Permission::SuperAdmin | Permission::Admin)
    }
}

pub struct SuperAdminOnly;

impl PermissionPolicy for SuperAdminOnly {
    fn allows(permission: Permission) -> bool {
        matches!(permission, Permission::SuperAdmin)
    }
}

pub struct AuthManager<P = AnyPermission> {
    pub name: String,
    pub permission: Permission,
    _policy: PhantomData<P>,
}

impl<S, P> FromRequestParts<S> for AuthManager<P>
where
    S: Send + Sync,
    AppState: FromRef<S>,
    Key: FromRef<S>,
    P: PermissionPolicy,
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
        let mut db = app_state.db;

        match Manager::all()
            .filter(Manager::fields().name().eq(claims.name.clone()))
            .first()
            .exec(&mut db)
            .await
        {
            Ok(Some(manager)) => {
                let permission = manager.permission_enum();

                if !P::allows(permission) {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(json!({
                            "ok": false,
                            "reason": "权限不足",
                        })),
                    )
                        .into_response());
                }

                Ok(Self {
                    name: claims.name,
                    permission,
                    _policy: PhantomData,
                })
            }
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
