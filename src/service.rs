use axum::{
    Router, middleware,
    routing::{get, post},
};

use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::handler;
use crate::{AppState, error::AppErr};

pub async fn api_router(state: AppState) -> Result<Router, AppErr> {
    //manager route
    let manager_route = Router::new()
        .route("/", get(handler::manager::list_manager))
        .route(
            "/{id}",
            post(handler::manager::add_manager)
                .delete(handler::manager::del_manager)
                .get(handler::permission::get_password)
                .patch(handler::manager::refresh_password_manager),
        );

    let permisson_route =
        Router::new().route("/{id}", get(handler::permission::handle_get_permisson));

    let cors = CorsLayer::permissive();
    let api = Router::new()
        .merge(common_route_bundle()) //一些通用接口
        .nest("/metrics", metric_route_bundle()) //放layer后面防止统计
        .route("/info", get(handler::info::get_stranger_info))
        .nest("/permission", permisson_route)
        .route(
            "/auth",
            post(handler::webui::auth).get(handler::webui::is_login),
        )
        .nest("/manager", manager_route)
        .route("/list", get(handler::banmanagement::list))
        .route(
            "/{id}",
            get(handler::api::check)
                .post(handler::banmanagement::ban)
                .delete(handler::banmanagement::unban),
        )
        .route(
            "/report/{id}",
            get(handler::banmanagement::get_report)
                .post(handler::banmanagement::report)
                .delete(handler::banmanagement::clean_report),
        );

    let memory_router = memory_serve::load!()
        .index_file(Some("/index.html"))
        .into_router();

    let app = Router::new()
        .nest("/api", api)
        .merge(memory_router)
        .layer(cors)
        .layer(CompressionLayer::new())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::record_api,
        )) //放这里是为了不记录metric的请求
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::record_request,
        )) //记录所有请求
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().no_deflate())
        .with_state(state);
    Ok(app)
}

fn common_route_bundle() -> Router<AppState> {
    Router::new()
        .route("/version", get(handler::version))
        .route("/health", get(handler::health::health_check))
}

fn metric_route_bundle() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::metrics::all_metrics))
        .route("/success", get(handler::metrics::success))
        .route("/fail", get(handler::metrics::fail))
        .route(
            "/banned",
            get(handler::banmanagement::banned_user_count_handle),
        )
        .route("/request", get(handler::metrics::all_request))
        .route("/sse", get(handler::metrics::sse))
        .route("/list", get(handler::metrics::list))
}
