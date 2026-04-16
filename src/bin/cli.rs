use anyhow::Context;
use toasty_cli::{Config, ToastyCli};
use qq_banner::{DATA_DIR, DB_PATH};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().context(
        "failed to load Toasty config file; expected Toasty.toml/toasty.toml in current working directory",
    )?;
    std::fs::create_dir_all(Path::new(DATA_DIR))?;

    let db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&format!("sqlite:{}/{}", DATA_DIR, DB_PATH))
        .await?;

    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;

    Ok(())
}
