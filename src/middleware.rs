use std::sync::atomic::Ordering;

use axum::{extract::Request, middleware::Next, response::Response};
use qq_banner::{METRIC_FAIL, METRIC_REQUEST, METRIC_SUCCESS};

/// 记录api请求，有路由匹配才记录
pub async fn record_api(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success() {
        METRIC_SUCCESS.fetch_add(1, Ordering::Relaxed);
    } else {
        METRIC_FAIL.fetch_add(1, Ordering::Relaxed);
    }
    response
}

/// 记录所有请求，不管有没有路由匹配
pub async fn record_request(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    METRIC_REQUEST.fetch_add(1, Ordering::Relaxed);
    response
}
