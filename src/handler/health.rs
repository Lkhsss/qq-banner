use qq_banner::model::Manager;

use super::*;

pub async fn health_check(State(state): State<AppState>) -> String {
    let mut db = state.db;
    let mut health = 0.;
    let mut count = 0;
    let database_check = Manager::filter_by_name("admin").first().exec(&mut db).await;
    match database_check {
        Ok(d) => {
            count += 1;
            if d.is_some() {
                count += 1;
            }
        }
        Err(_) => (),
    }
    health = count as f64 / 2.0;
    health.to_string()
}
