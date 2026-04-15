use axum::{Json, response::IntoResponse};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, toasty::Model, Serialize)]
pub struct User {
    #[key]
    pub id: u64,
    pub time: u64,
}

impl IntoResponse for User {
    fn into_response(self) -> axum::response::Response {
        Json(json!(self)).into_response()
    }
}
