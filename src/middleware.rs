use axum::{
    Router,
    extract::{Request, State},
    http,
    middleware::{self, Next},
    response::Response,
    routing::get,
};

use crate::AppState;
//记录api请求，有路由匹配才记录
pub async fn record_api(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let metrics = state.metrics;
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success() {
        let _ = metrics.fetch_and_update(b"counter:success", |old| {
            let n = old
                .map(|b| u64::from_be_bytes(b.try_into().unwrap())) //FIXME
                .unwrap_or(0);
            Some((n + 1).to_be_bytes().to_vec())
        });
    } else {
        let _ = metrics.fetch_and_update(b"counter:fail", |old| {
            let n = old
                .map(|b| u64::from_be_bytes(b.try_into().unwrap())) //FIXME
                .unwrap_or(0);
            Some((n + 1).to_be_bytes().to_vec())
        });
    }
    response
}

//记录所有请求，不管有没有路由匹配
pub async fn record_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let metrics = state.metrics;
    let response = next.run(request).await;
    let _ = metrics.fetch_and_update(b"counter:request", |old| {
        let n = old
            .map(|b| u64::from_be_bytes(b.try_into().unwrap())) //FIXME
            .unwrap_or(0);
        Some((n + 1).to_be_bytes().to_vec())
    });
    response
}
