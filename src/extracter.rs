use axum::{
    Form, Json,
    extract::{FromRef, FromRequest, FromRequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use jsonwebtoken::{DecodingKey, Validation, decode};
use qq_banner::SALT;
use qq_banner::model::{Manager, Permission};
use serde_json::json;
use std::marker::PhantomData;

use crate::{AppState, error::AppErr, handler::Claim};

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

impl<S, P> FromRequest<S> for AuthManager<P>
where
    S: Send + Sync,
    AppState: FromRef<S>,
    Key: FromRef<S>,
    P: PermissionPolicy,
{
    type Rejection = Response;

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let (mut parts, body) = req.into_parts();
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
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "ok": false,
                        "reason": "权限不足",
                    })),
                )
                    .into_response());
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
            .map_err(|err| (StatusCode::UNAUTHORIZED, err).into_response())?;

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
                    return Err((StatusCode::FORBIDDEN, "权限不足").into_response());
                }
                Ok(Self {
                    name: manager.name,
                    permission,
                    _policy: PhantomData,
                })
            }
            Ok(None) => Err((StatusCode::UNAUTHORIZED, "用户名或密码错误").into_response()),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("数据库错误: {err}"),
            )
                .into_response()),
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

/// 从PrivateCookieJar中提取token并解析出管理员信息，返回Manager结构体
#[deprecated]
pub async fn valid_cookie(jar: PrivateCookieJar<Key>, state: AppState) -> Result<Manager, AppErr> {
    let token_cookie = jar.get("token").ok_or(AppErr::TokenMissing)?;
    let token = token_cookie.value();

    let claims = decode::<Claim>(
        token,
        &DecodingKey::from_secret(SALT.as_bytes()),
        &Validation::default(),
    )
    .map_err(AppErr::TokenInvalid)?
    .claims;

    let mut db = state.db;
    let manager = Manager::all()
        .filter(Manager::fields().name().eq(claims.name.clone()))
        .first()
        .exec(&mut db)
        .await?
        .ok_or(AppErr::UserNotFound)?;

    Ok(manager)
}
