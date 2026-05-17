use super::*;
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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppErr,
    extracter::{AdminOrAbove, AuthManager, SuperAdminOnly},
    handler::{Claim, UserStatusBack, is_ban_expired, now_unix_secs, permission::get_permisson},
};

use crate::database::Banner;

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
    let q = Manager::filter_by_name(&name).first().exec(&mut db).await?;

    if q.is_some() {
        return Err(AppErr::ManagerExists);
    }
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

pub async fn refresh_password_manager(
    _auth: AuthManager<SuperAdminOnly>,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Manager>, AppErr> {
    let mut db = state.db;
    let password = Uuid::new_v4().simple().to_string();

    Manager::filter(Manager::fields().name().eq(&name))
        .update()
        .password(password)
        .exec(&mut db)
        .await?;
    let manager = Manager::filter(Manager::fields().name().eq(&name))
        .first()
        .exec(&mut db)
        .await?;

    match manager {
        Some(m) => Ok(Json(m)),
        None => Err(AppErr::Database_Unhealth),
    }
}

pub async fn del_manager(
    _auth: AuthManager<SuperAdminOnly>,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppErr> {
    println!("删除管理账号:{}", name);
    if name == "admin" {
        return Err(AppErr::PermissonDenied);
    }
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
        //自增减1
        METRIC_BANNED.fetch_sub(1, Ordering::Relaxed);
    }
    println!("webui: id: [{}]解除封禁", id);
    Ok(Json(UserStatusBack::unbanned(id)))
}

pub async fn ban(
    operator: AuthManager<AdminOrAbove>,
    Path(id): Path<u64>,
    Query(params): Query<BanQuery>,
    State(state): State<AppState>,
) -> Result<Json<UserStatusBack>, AppErr> {
    let timestamp_secs = now_unix_secs();
    let mut db = state.db;
    //检查操作人权限
    let permisson = get_permisson(&mut db, &id.to_string()).await?;

    if operator.permission <= permisson.into() {
        return Err(AppErr::PermissonDenied);
    }

    let new_user = User {
        id,
        time: timestamp_secs,
        duration: params.duration.unwrap_or(0),
        operator: operator.name,
    };
    let banner = db.ban(new_user).await?;

    println!("Banned QQ : {}", banner.id);
    Ok(Json(banner))
}

#[derive(Debug, Deserialize)]
pub struct BanQuery {
    pub duration: Option<u64>,
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
