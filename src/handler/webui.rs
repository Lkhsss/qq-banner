use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use qq_banner::{
    SALT,
    model::{Manager, User},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppErr,
    handler::{AuthManager, UserStatusBack},
};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    // let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(SALT.as_bytes())
});

pub async fn list_manager(
    _auth: AuthManager,
    State(state): State<AppState>,
) -> Result<Json<Vec<Manager>>, AppErr> {
    let mut db = state.0;
    let users = Manager::all().exec(&mut db).await?;
    Ok(Json(users))
}

pub async fn add_manager(
    _auth: AuthManager,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.0;
    let password = Uuid::new_v4().simple().to_string();
    let manager = toasty::create!(Manager { name, password })
        .exec(&mut db)
        .await?;

    Ok(Json(manager))
}

pub async fn del_manager(
    _auth: AuthManager,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    println!("删除管理账号:{}", name);
    let mut db = state.0;

    Manager::filter_by_name(&name)
        .delete()
        .exec(&mut db)
        .await?;

    Ok(name)
}

pub async fn qq_userinfo(Path(qq): Path<u64>) -> Result<Json<Value>, AppErr> {
    let url = format!("https://uapis.cn/api/v1/social/qq/userinfo?qq={qq}");
    let response = reqwest::Client::new()
        .get(url)
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await?
        .error_for_status()?;

    let payload = response.json::<Value>().await?;
    Ok(Json(payload))
}

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
#[derive(Debug, Serialize, Deserialize)]
struct Claim {
    name: String,
    exp: i64,
}
pub async fn auth(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(manager): Form<Manager>,
) -> Result<PrivateCookieJar, AppErr> {
    let mut db = state.0;

    let manager_valid = Manager::all()
        .filter(Manager::fields().name().eq(&manager.name))
        .filter(Manager::fields().password().eq(manager.password))
        .first()
        .exec(&mut db)
        .await?;

    if manager_valid.is_none() {
        return Err(AppErr::BadPassword);
    }
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(qq_banner::EXPIRE_TIME))
        .expect("valid timestamp")
        .timestamp();
    let claim = Claim {
        name: manager.name,
        exp: expiration,
    };
    let access_token = encode(&Header::default(), &claim, &KEYS.encoding)?;
    let cookie = Cookie::build(("token", access_token))
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true);

    Ok(jar.add(cookie))
}

pub async fn is_login(jar: PrivateCookieJar) -> impl IntoResponse {
    let Some(token_cookie) = jar.get("token") else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "reason": "未登录：缺少token",
            })),
        );
    };

    let token = token_cookie.value();
    match decode::<Claim>(token, &KEYS.decoding, &Validation::default()) {
        Ok(data) => {
            println!("{} 验证登录", data.claims.name);
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "name": data.claims.name,
                })),
            )
        }
        Err(err) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "reason": format!("token校验失败: {err}"),
            })),
        ),
    }
}

pub async fn unban(
    _: AuthManager,
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.0;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;

    if let Some(u) = users {
        u.delete().exec(&mut db).await?;
    }
    println!("webui: id: [{}]解除封禁", id);
    Ok(Json(UserStatusBack::unbanned(id)))
}

pub async fn ban(
    _: AuthManager,
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp_secs = since_the_epoch.as_secs();

    let mut db = state.0;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;
    //存在则直接返回
    match users {
        Some(u) => {
            println!("id: [{}] already banned", id);
            return Ok(Json(UserStatusBack::banned(u)));
        }
        _ => (),
    }
    let user = toasty::create!(User {
        id,
        time: timestamp_secs,
    })
    .exec(&mut db)
    .await?;
    println!("Banned QQ : {}", user.id);
    Ok(Json(UserStatusBack::banned(user)))
}
