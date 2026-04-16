use anyhow::{Context, Result};
use axum::{
    Router,
    routing::{get, post},
};
use qq_banner::{ADDR, DATA_DIR, DB_PATH, PORT, model::Manager};
use std::path::Path;
use toasty::Db;
use toasty_cli::{Config, ToastyCli};
use uuid::Uuid;
mod error;
mod handler;

#[tokio::main]
async fn main() -> Result<()> {
    std::fs::create_dir_all(Path::new(DATA_DIR))?;
    let db_url = format!("sqlite:{}/{}", DATA_DIR, DB_PATH);

    // 启动服务前自动应用待执行迁移。
    let migration_db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&db_url)
        .await?;
    let migration_config = Config::load().context(
        "failed to load Toasty config file; expected Toasty.toml/toasty.toml in current working directory",
    )?;
    let migration_cli = ToastyCli::with_config(migration_db, migration_config);
    migration_cli
        .parse_from(["qq-banner", "migration", "apply"])
        .await?;

    // 加载配置
    let mut db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&db_url)
        .await?;

    let random_password = Uuid::new_v4().simple().to_string();
    let admin = Manager::all()
        .filter(Manager::fields().name().eq("admin".to_string()))
        .first()
        .exec(&mut db)
        .await?;

    let admin_password = match admin {
        Some(manager) => manager.password,
        None => {
            let manager = toasty::create!(Manager {
                name: "admin".to_string(),
                password: random_password,
            })
            .exec(&mut db)
            .await?;
            manager.password
        }
    };

    println!("管理员账号：admin");
    println!("管理员密码：{}", admin_password);

    let state = AppState(db);

    let app = Router::new()
        .route("/unban/{id}", post(handler::unban))
        .route("/list", get(handler::list))
        .route("/{id}", post(handler::ban).get(handler::check))
        .with_state(state);

    println!("服务已启动！");
    println!("监听位置：{}", format_args!("{ADDR}:{PORT}"));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", ADDR, PORT)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
#[derive(Clone)]
struct AppState(Db);
