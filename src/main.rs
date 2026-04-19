use anyhow::{Context, Result};
use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};
use axum_extra::extract::cookie::Key;
use qq_banner::{ADDR, API_PORT, DATA_DIR, DB_PATH, DIST_DIR, PROJECT_DIR, model::Manager};
use std::path::{Path, PathBuf};
use toasty::Db;
use toasty_cli::{Config, ToastyCli};
use uuid::Uuid;
mod error;
mod handler;
mod service;

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



    //释放前端目录
    PROJECT_DIR
        .extract(PathBuf::from(DIST_DIR))
        .expect("无法提取项目目录");

    let state = AppState(db, Key::generate());

    let (api_res, webui_res) = tokio::join!(
        service::api_service(state.clone()),
        service::webui_service(state)
    );
    api_res?;
    webui_res?;
    Ok(())
}
#[derive(Clone)]
struct AppState(Db, Key);

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.1.clone()
    }
}
