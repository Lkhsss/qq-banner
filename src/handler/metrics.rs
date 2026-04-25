use crate::AppState;
use crate::error::AppErr;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};
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
