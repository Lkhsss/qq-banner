use std::sync::atomic::Ordering;

use axum::http::header;
use axum::{extract::Request, middleware::Next, response::Response};
use qq_banner::{DAY_FAIL, DAY_REQUEST, DAY_SUCCESS, METRIC_FAIL, METRIC_REQUEST, METRIC_SUCCESS};

/// 记录api请求，有路由匹配才记录
pub async fn record_api(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success() {
        METRIC_SUCCESS.fetch_add(1, Ordering::Relaxed);
        DAY_SUCCESS.fetch_add(1, Ordering::Relaxed);
    } else {
        METRIC_FAIL.fetch_add(1, Ordering::Relaxed);
        DAY_FAIL.fetch_add(1, Ordering::Relaxed);
    }
    response
}

/// 记录所有请求，不管有没有路由匹配
pub async fn record_request(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    METRIC_REQUEST.fetch_add(1, Ordering::Relaxed);
    DAY_REQUEST.fetch_add(1, Ordering::Relaxed);
    response
}

/// 添加安全响应头，防止点击劫持、MIME 嗅探、XSS、中间人攻击
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_FRAME_OPTIONS,
        header::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        header::HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ws: wss:",
        ),
    );
    response
}

/// HSTS 中间件：将 HTTP 重定向到 HTTPS（生产环境中启用）
pub async fn hsts_redirect(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::STRICT_TRANSPORT_SECURITY,
        header::HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    response
}
