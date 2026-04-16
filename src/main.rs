use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use qq_banner::DB_PATH;
use std::path::Path;
use toasty::Db;
use toasty_cli::{Config, ToastyCli};
mod error;
mod handler;

const addr: &str = "0.0.0.0";
const port: &str = "6100";
#[tokio::main]
async fn main() -> Result<()> {
    std::fs::create_dir_all(Path::new("./data"))?;
    let db_url = format!("sqlite:./data/{}", DB_PATH);

    // 启动服务前自动应用待执行迁移。
    let migration_db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&db_url)
        .await?;
    let migration_config = Config::load()?;
    let migration_cli = ToastyCli::with_config(migration_db, migration_config);
    migration_cli
        .parse_from(["qq-banner", "migration", "apply"])
        .await?;

    // 加载配置
    let mut db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&db_url)
        .await?;

    let state = AppState(db);

    let app = Router::new()
        .route("/{id}", post(handler::ban).get(handler::check))
        .route("/list", get(handler::list))
        .with_state(state);

    println!("服务已启动！");
    println!("监听位置：{}", format_args!("{addr}:{port}"));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", addr, port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
#[derive(Clone)]
struct AppState(Db);
