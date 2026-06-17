use axum::{
    Form,
    extract::{FromRef, FromRequest, FromRequestParts, OriginalUri},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use jsonwebtoken::{DecodingKey, Validation, decode};
use qq_banner::SALT;
use qq_banner::model::{Manager, Permission};
use std::marker::PhantomData;

use crate::{AppState, error::AppErr, handler::Claim};
use tracing::error;

pub trait PermissionPolicy {
    fn allows(permission: Permission) -> bool;
}

#[derive(Debug)]
pub struct AnyPermission;

impl PermissionPolicy for AnyPermission {
    fn allows(_: Permission) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct AdminOrAbove;

impl PermissionPolicy for AdminOrAbove {
    fn allows(permission: Permission) -> bool {
        matches!(permission, Permission::SuperAdmin | Permission::Admin)
    }
}

#[derive(Debug)]
pub struct UserOrAbove;

impl PermissionPolicy for UserOrAbove {
    fn allows(permission: Permission) -> bool {
        matches!(
            permission,
            Permission::SuperAdmin | Permission::Admin | Permission::User
        )
    }
}

#[derive(Debug)]
pub struct SuperAdminOnly;

impl PermissionPolicy for SuperAdminOnly {
    fn allows(permission: Permission) -> bool {
        matches!(permission, Permission::SuperAdmin)
    }
}

#[derive(Debug)]
pub struct AuthManager<P = AnyPermission> {
    pub name: String,
    pub permission: Permission,
    _policy: PhantomData<P>,
}

impl<S, P> FromRequest<S> for AuthManager<P>
where
    S: Send + Sync,
    AppState: FromRef<S>,
    Key: FromRef<S>,
    P: PermissionPolicy,
{
    type Rejection = AppErr;

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let (mut parts, body) = req.into_parts();
        let uri = OriginalUri::from_request_parts(&mut parts, state)
            .await
            .map(|u| u.0.to_string())
            .unwrap_or_else(|_| parts.uri.to_string());
        // ── 快速路径：尝试 Cookie 鉴权 ──
        // PrivateCookieJar::from_request_parts 的 Err 类型是 Infallible，
        // 类型系统保证此处不可能失败，无需 unwrap/expect。
        #[allow(irrefutable_let_patterns)]
        let Ok(jar) = PrivateCookieJar::<Key>::from_request_parts(&mut parts, state).await;

        let mut db = app_state.db;

        if let Some(token_cookie) = jar.get("token")
            && let Ok(manager) = try_cookie(token_cookie.value(), &mut db).await
        {
            let permission = manager.permission_enum();
            if !P::allows(permission) {
                error!(
                    uri = %uri,
                    user = %manager.name,
                    reason = "权限不足",
                    method = "cookie",
                );
                return Err(AppErr::PermissonDenied);
            }
            return Ok(Self {
                name: manager.name,
                permission,
                _policy: PhantomData,
            });
        }

        // ── 回退路径：Form 用户名+密码鉴权 ──
        let form_req = axum::http::Request::from_parts(parts, body);
        let Form(manager): Form<Manager> = Form::from_request(form_req, state)
            .await
            .map_err(|err| AppErr::LoginErr(err.to_string()))?;

        match Manager::all()
            .filter(Manager::fields().name().eq(&manager.name))
            .filter(Manager::fields().password().eq(&manager.password))
            .first()
            .exec(&mut db)
            .await
        {
            Ok(Some(manager)) => {
                let permission = manager.permission_enum();
                if !P::allows(permission) {
                    error!(
                        uri = %uri,
                        user = %manager.name,
                        reason = "权限不足",
                        method = "form",
                    );
                    return Err(AppErr::PermissonDenied);
                }
                Ok(Self {
                    name: manager.name,
                    permission,
                    _policy: PhantomData,
                })
            }
            Ok(None) => {
                error!(
                    uri = %uri,
                    user = %manager.name,
                    reason = "用户名或密码错误",
                    method = "form",
                );
                Err(AppErr::BadPassword)
            }
            Err(err) => Err(AppErr::Database(err)),
        }
    }
}

/// Cookie 鉴权内部辅助函数：解码 JWT 并查 DB，返回 Manager
async fn try_cookie(token: &str, db: &mut toasty::Db) -> Result<Manager, AppErr> {
    let claims = decode::<Claim>(
        token,
        &DecodingKey::from_secret(SALT.as_bytes()),
        &Validation::default(),
    )
    .map_err(AppErr::TokenInvalid)?
    .claims;

    Manager::all()
        .filter(Manager::fields().name().eq(claims.name))
        .first()
        .exec(db)
        .await?
        .ok_or(AppErr::UserNotFound)
}

// 从PrivateCookieJar中提取token并解析出管理员信息，返回Manager结构体
// #[deprecated]
// pub async fn valid_cookie(jar: PrivateCookieJar<Key>, state: AppState) -> Result<Manager, AppErr> {
//     let token_cookie = jar.get("token").ok_or(AppErr::TokenMissing)?;
//     let token = token_cookie.value();

//     let claims = decode::<Claim>(
//         token,
//         &DecodingKey::from_secret(SALT.as_bytes()),
//         &Validation::default(),
//     )
//     .map_err(AppErr::TokenInvalid)?
//     .claims;

//     let mut db = state.db;
//     let manager = Manager::all()
//         .filter(Manager::fields().name().eq(claims.name.clone()))
//         .first()
//         .exec(&mut db)
//         .await?
//         .ok_or(AppErr::UserNotFound)?;

//     Ok(manager)
// }
