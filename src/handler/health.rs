use std::time::Duration;

use axum::Json;
use qq_banner::{NAPCAT_ADDR, NAPCAT_PORT, NAPCAT_TOKEN, model::Manager};

use tracing::info;
use tracing::instrument;

use super::*;
#[derive(Default, Debug, Serialize)]
pub struct Health {
    database: bool,
    napcat_online: bool,
    napcat_good: bool,
    health: f64,
}

impl Health {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn calc(&mut self) {
        let mut count = 0;
        if self.database {
            count += 1;
        }
        if self.napcat_online {
            count += 1;
        }
        if self.napcat_good {
            count += 1;
        }
        self.health = count as f64 / 3.;
    }
}
#[derive(Deserialize)]
pub struct NapcatStatusResponse {
    data: NapcatStatus,
}

#[derive(Deserialize)]
pub struct NapcatStatus {
    online: bool,
    good: bool,
}
/// # 检查健康度
#[instrument(name = "健康检查", skip(state))]
pub async fn health_check(State(state): State<AppState>) -> Result<Json<Health>, AppErr> {
    let mut db = state.db;
    let mut health = Health::new();

    let database_check = Manager::filter_by_name("admin").first().exec(&mut db).await;

    if let Ok(d) = database_check
        && d.is_some()
    {
        health.database = true;
    }

    let response = reqwest::Client::new()
        .post(format!("{}:{}/get_status", NAPCAT_ADDR, NAPCAT_PORT))
        .bearer_auth(NAPCAT_TOKEN)
        .timeout(Duration::from_secs(1))
        .send()
        .await;
    match response {
        Ok(o) => {
            let napcat: NapcatStatusResponse = o.json().await.unwrap_or(NapcatStatusResponse {
                data: NapcatStatus {
                    online: false,
                    good: false,
                },
            });
            health.napcat_online = napcat.data.online;
            health.napcat_good = napcat.data.good;
        }
        Err(_) => {
            health.napcat_online = false;
            health.napcat_good = false;
        }
    }

    health.calc();
    info!(
        "健康度: {:.0}% (DB:{}, NC:{}, NG:{})",
        health.health * 100.0,
        health.database as u8,
        health.napcat_online as u8,
        health.napcat_good as u8
    );
    Ok(Json(health))
}
