use axum::{
    Router,
    routing::{get, post},
};
use qq_banner::{globals::WEBUI_PORT, *};
use tower_http::services::{ServeDir, ServeFile};

use crate::handler;
use crate::{AppState, error::AppErr};



pub async fn api_service(state: AppState) -> Result<(), AppErr> {
    println!("api服务已启动！");
    println!("监听位置：{}", format_args!("{ADDR}:{API_PORT}"));
    let app = Router::new()
        .route("/api/list", get(handler::list))
        .route(
            "/api/{id}",
            post(handler::api::ban)
                .get(handler::api::check)
                .delete(handler::api::unban),
        )
        .route("/api/version", get(handler::version))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ADDR, API_PORT)).await?;
    Ok(axum::serve(listener, app).await?)
}

pub async fn webui_service(state: AppState) -> Result<(), AppErr> {
    println!("webui服务已启动！");
    println!("监听位置：{}", format_args!("{ADDR}:{WEBUI_PORT}"));
    // Vue history 路由在找不到真实文件时回退到 index.html。
    let web_assets = ServeDir::new(DIST_DIR)
        .not_found_service(ServeFile::new(format!("{DIST_DIR}/index.html")));
    //manager route
    let manager_route = Router::new()
        .route("/", get(handler::webui::list_manager))
        .route(
            "/{name}",
            post(handler::webui::add_manager).delete(handler::webui::del_manager),
        );

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/version", get(handler::version))
                .route(
                    "/auth",
                    post(handler::webui::auth).get(handler::webui::is_login),
                )
                .route("/qq/userinfo/{id}", get(handler::webui::qq_userinfo))
                .route("/list", get(handler::list))
                .nest("/manager", manager_route)
                .route(
                    "/{id}",
                    post(handler::webui::ban).delete(handler::webui::unban),
                ),
        )
        .fallback_service(web_assets)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ADDR, WEBUI_PORT)).await?;
    Ok(axum::serve(listener, app).await?)
}
