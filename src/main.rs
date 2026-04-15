use anyhow::Result;
use axum::{Router, routing::post};
use std::path::Path;
use toasty::Db;
mod error;
mod handler;
mod model;
const addr: &str = "0.0.0.0";
const port: &str = "6100";
const db_path: &str = "./namelist.sqlite";
#[tokio::main]
async fn main() -> Result<()> {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&format!("sqlite:{}", db_path))
        .await?;

    if !Path::new(db_path).exists() {
        db.push_schema().await?;
    }

    let state = AppState(db);

    let app = Router::new()
        .route("/{id}", post(handler::ban).get(handler::check))
        .with_state(state);

    println!("服务已启动！");
    println!("监听位置：{}", format_args!("{addr}:{port}"));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", addr, port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
#[derive(Clone)]
struct AppState(Db);
