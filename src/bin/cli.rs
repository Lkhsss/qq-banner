use toasty_cli::{Config, ToastyCli};
use qq_banner::DB_PATH;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    std::fs::create_dir_all(Path::new("./data"))?;

    let db = toasty::Db::builder()
        .models(toasty::models!(qq_banner::*))
        .connect(&format!("sqlite:./data/{}", DB_PATH))
        .await?;

    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;

    Ok(())
}
