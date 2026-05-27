use axum::{
    Router,
    http::HeaderValue,
    middleware,
    routing::{get, post},
};
use qq_banner::{globals::WEBUI_PORT, *};
use reqwest::Method;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::handler;
use crate::{AppState, error::AppErr};

pub async fn api_service(state: AppState) -> Result<(), AppErr> {
    println!("api服务已启动！");
    println!("监听位置：{}", format_args!("{ADDR}:{API_PORT}"));

    let permisson_route = Router::new().route(
        "/{id}",
        get(handler::permission::handle_get_permisson).post(handler::permission::check_permisson),
    );
    let manager_route = Router::new().route(
        "/{id}",
        post(handler::api::add_manager)
            .delete(handler::api::del_manager)
            .get(handler::permission::get_password),
    );

    let app = Router::new()
        .nest("/api", common_route())
        .nest("/api/permission", permisson_route)
        .nest("/api/manager", manager_route)
        .route(
            "/api/{id}",
            post(handler::banmanagement::ban)
                .get(handler::api::check)
                .delete(handler::banmanagement::unban),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::record_api,
        )) //放这里是为了不记录metric的请求
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::record_request,
        )) //记录所有请求
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ADDR, API_PORT)).await?;
    Ok(axum::serve(listener, app).await?)
}

pub async fn webui_service(state: AppState) -> Result<(), AppErr> {
    println!("webui服务已启动！");
    println!("监听位置：{}", format_args!("{ADDR}:{WEBUI_PORT}"));
    // Vue history 路由在找不到真实文件时回退到 index.html
    // let web_assets = ServeDir::new(DIST_DIR);
    // .not_found_service(ServeFile::new(format!("{DIST_DIR}/index.html")));


    //manager route
    let manager_route = Router::new()
        .route("/", get(handler::webui::list_manager))
        .route(
            "/{name}",
            post(handler::webui::add_manager)
                .delete(handler::webui::del_manager)
                .patch(handler::webui::refresh_password_manager),
        );
    let permisson_route =
        Router::new().route("/{id}", get(handler::permission::handle_get_permisson));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_origin(Any);

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(common_route())
                .nest("/permission", permisson_route)
                .route(
                    "/auth",
                    post(handler::webui::auth).get(handler::webui::is_login),
                )
                .route("/qq/userinfo/{id}", get(handler::webui::qq_userinfo))
                .nest("/manager", manager_route)
                //ban/unban删除
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    crate::middleware::record_api,
                ))
                .nest("/metrics", metric_route()), //放layer后面防止统计,
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::record_request,
        )) //记录所有请求
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ADDR, WEBUI_PORT)).await?;
    Ok(axum::serve(listener, app).await?)
}

fn common_route() -> Router<AppState> {
    Router::new()
        .route("/list", get(handler::list))
        .route("/list/count", get(handler::banned_user_count_handle))
        .route("/version", get(handler::version))
        .route("/health", get(handler::health::health_check))
}

fn metric_route() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::metrics::all_metrics))
        .route("/success", get(handler::metrics::success))
        .route("/fail", get(handler::metrics::fail))
        .route("/banned", get(handler::banned_user_count_handle))
        .route("/request", get(handler::metrics::all_request))
        .route("/sse", get(handler::metrics::sse))
}
