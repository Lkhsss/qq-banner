use crate::AppState;
use crate::error::AppErr;
use crate::handler::banned_user_count;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;
use sled::IVec;

pub async fn success(State(state): State<AppState>) -> Result<String, AppErr> {
    let db = state.metrics;
    let data = db
        .get(b"counter:success")?
        .unwrap_or(IVec::from(0_u64.to_be_bytes().to_vec()));

    Ok(u64::from_be_bytes(data.as_ref().try_into()?).to_string())
}

pub async fn fail(State(state): State<AppState>) -> Result<String, AppErr> {
    let db = state.metrics;
    let data = db
        .get(b"counter:fail")?
        .unwrap_or(IVec::from(0_u64.to_be_bytes().to_vec()));
    Ok(u64::from_be_bytes(data.as_ref().try_into()?).to_string())
}

pub async fn all_request(State(state): State<AppState>) -> Result<String, AppErr> {
    let db = state.metrics;
    let data = db
        .get(b"counter:request")?
        .unwrap_or(IVec::from(0_u64.to_be_bytes().to_vec()));
    Ok(u64::from_be_bytes(data.as_ref().try_into()?).to_string())
}

pub async fn all_metrics(State(state): State<AppState>) -> Result<Metrics, AppErr> {
    let m_db = state.metrics;
    let success = get_metric("counter:success", &m_db).await?;
    let fail = get_metric("counter:fail", &m_db).await?;
    let request = get_metric("counter:request", &m_db).await?;

    // 计算被封禁的人数
    let mut db = state.db;
    let banned = banned_user_count(&mut db).await?;

    Ok(Metrics {
        success,
        fail,
        request,
        banned,
    })
}

/// 指标聚合
#[derive(Debug, Serialize)]
pub struct Metrics {
    success: u64,
    fail: u64,
    request: u64,
    banned: usize,
}

pub async fn get_metric(name: &str, db: &sled::Db) -> Result<u64, AppErr> {
    let data = db
        .get(name.as_bytes())?
        .unwrap_or(IVec::from(0_u64.to_be_bytes().to_vec()));
    Ok(u64::from_be_bytes(data.as_ref().try_into()?))
}

impl IntoResponse for Metrics {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
