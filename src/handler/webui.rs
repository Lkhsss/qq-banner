use std::sync::LazyLock;

use axum::{Form, Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{
    CookieJar, PrivateCookieJar,
    cookie::{Cookie, SameSite},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use qq_banner::{SALT, model::Manager};

use serde::Serialize;

use crate::{AppState, error::AppErr, handler::Claim};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    // let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(SALT.as_bytes())
});

// pub async fn del_manager(
//     Path(name): Path<String>,
//     State(state): State<AppState>,
//     _auth: AuthManager<SuperAdminOnly>,
// ) -> Result<String, AppErr> {
//     println!("删除管理账号:{}", name);
//     if name == "admin" {
//         return Err(AppErr::PermissonDenied);
//     }
//     let mut db = state.db;

//     Manager::filter_by_name(&name)
//         .delete()
//         .exec(&mut db)
//         .await?;

//     Ok(name)
// }

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}
impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// # 登录
/// form登录
pub async fn auth(
    State(state): State<AppState>,
    private_jar: PrivateCookieJar,
    jar: CookieJar,
    Form(manager): Form<Manager>,
) -> Result<(PrivateCookieJar, CookieJar, ManagerInfo), AppErr> {
    println!("用户：{} 鉴权", manager.name);
    let mut db = state.db;

    let manager_valid = Manager::all()
        .filter(Manager::fields().name().eq(&manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;

    let manager_valid = match manager_valid {
        Some(m) => m,
        None => return Err(AppErr::BadPassword),
    };

    let manager_info: ManagerInfo = ManagerInfo {
        name: manager_valid.name,
        permission: manager_valid.permission,
    };
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(qq_banner::EXPIRE_TIME))
        .expect("valid timestamp")
        .timestamp();
    let claim = Claim {
        name: manager.name,
        exp: expiration,
    };
    let access_token = encode(&Header::default(), &claim, &KEYS.encoding)?;
    let cookie_token = Cookie::build(("token", access_token))
        .path("/")
        .same_site(SameSite::Strict)
        .http_only(true);
    let cookie_permisson =
        Cookie::build(("permisson", manager_info.permission.to_string())).path("/");

    let cookie_name = Cookie::build(("name", manager_info.name.clone())).path("/");

    Ok((
        private_jar.add(cookie_token),
        jar.add(cookie_permisson).add(cookie_name),
        manager_info,
    ))
}

/// # Cookie验证登录
pub async fn is_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppErr> {
    let mut db = state.db;
    let Some(token_cookie) = jar.get("token") else {
        return Err(AppErr::LoginErr("缺少token".into()));
    };

    let token = token_cookie.value();
    match decode::<Claim>(token, &KEYS.decoding, &Validation::default()) {
        Ok(data) => {
            let manger = Manager::filter_by_name(data.claims.name)
                .one()
                .exec(&mut db)
                .await?;
            Ok((
                StatusCode::OK,
                ManagerInfo {
                    name: manger.name,
                    permission: manger.permission,
                },
            ))
        }
        Err(err) => return Err(AppErr::LoginErr(format!("token错误: {}", err))),
    }
}

#[derive(Serialize, Clone)]
pub struct ManagerInfo {
    pub name: String,
    pub permission: i16,
}

impl IntoResponse for ManagerInfo {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
