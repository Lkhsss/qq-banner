use qq_banner::model::Manager;

use super::*;

pub async fn health_check(State(state): State<AppState>) -> String {
    let mut db = state.db;
    let mut count = 0;
    let database_check = Manager::filter_by_name("admin").first().exec(&mut db).await;

    if let Ok(d) = database_check {
        count += 1;
        if d.is_some() {
            count += 1;
        }
    }
    let health = count as f64 / 2.0;
    health.to_string()
}
