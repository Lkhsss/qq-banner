use std::sync::LazyLock;

use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar, PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use qq_banner::{
    SALT,
    model::{Manager, Permission, User},
};

use serde_json::{Value, json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppErr,
    extracter::{AdminOrAbove, AuthManager, SuperAdminOnly},
    handler::{Claim, UserStatusBack, is_ban_expired, now_unix_secs},
};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    // let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(SALT.as_bytes())
});

pub async fn list_manager(
    _auth: AuthManager<SuperAdminOnly>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Manager>>, AppErr> {
    let mut db = state.db;
    let users = Manager::all().exec(&mut db).await?;
    Ok(Json(users))
}

/// 此处有自定义提取器验证身份，api需要自己调用数据库验证
pub async fn add_manager(
    _auth: AuthManager<SuperAdminOnly>,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.db;
    let password = Uuid::new_v4().simple().to_string();
    let manager = toasty::create!(Manager {
        name,
        password,
        permission: Permission::Admin as i16,
    })
    .exec(&mut db)
    .await?;

    Ok(Json(manager))
}

pub async fn del_manager(
    _auth: AuthManager<SuperAdminOnly>,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    println!("删除管理账号:{}", name);
    let mut db = state.db;

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

// 登录
pub async fn auth(
    State(state): State<AppState>,
    private_jar: PrivateCookieJar,
    jar: CookieJar,
    Form(manager): Form<Manager>,
) -> Result<(PrivateCookieJar, CookieJar), AppErr> {
    println!("用户：{} 鉴权",manager.name);
    let mut db = state.db;

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
    let cookie_token = Cookie::build(("token", access_token))
        .path("/")
        .same_site(SameSite::Strict)
        .http_only(true);
    let cookie_permisson =
        Cookie::build(("permisson", manager_valid.unwrap().permission.to_string())).path("/");

    Ok((private_jar.add(cookie_token), jar.add(cookie_permisson)))
}

/// 验证登录
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
        Ok(data) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "name": data.claims.name,
            })),
        ),
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
    _: AuthManager<AdminOrAbove>,
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let mut db = state.db;

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
    _: AuthManager<AdminOrAbove>,
    Path(id): Path<u64>,
    Query(params): Query<BanQuery>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let timestamp_secs = now_unix_secs();

    let mut db = state.db;

    let users = User::all()
        .filter(User::fields().id().eq(id))
        .first()
        .exec(&mut db)
        .await?;
    //存在则直接返回
    match users {
        Some(u) => {
            if is_ban_expired(&u, timestamp_secs) {
                u.delete().exec(&mut db).await?;
            } else {
                println!("id: [{}] already banned", id);
                return Ok(Json(UserStatusBack::banned(u)));
            }
        }
        _ => (),
    }
    let user = toasty::create!(User {
        id,
        time: timestamp_secs,
        duration: params.duration,
    })
    .exec(&mut db)
    .await?;
    println!("Banned QQ : {}", user.id);
    Ok(Json(UserStatusBack::banned(user)))
}

#[derive(Debug, Deserialize)]
pub struct BanQuery {
    #[serde(default)]
    pub duration: u64,
}
