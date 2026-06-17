use super::*;
use crate::database::Metrics;
use crate::error::AppErr;

use axum::Json;
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};

use futures_util::stream::{self, Stream};
use qq_banner::model;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use tracing::instrument;

#[instrument(name = "指标-成功数")]
pub async fn success() -> Result<String, AppErr> {
    Ok(METRIC_SUCCESS.load(Ordering::Relaxed).to_string())
}

#[instrument(name = "指标-失败数")]
pub async fn fail() -> Result<String, AppErr> {
    Ok(METRIC_FAIL.load(Ordering::Relaxed).to_string())
}

#[instrument(name = "指标-请求数")]
pub async fn all_request() -> Result<String, AppErr> {
    Ok(METRIC_REQUEST.load(Ordering::Relaxed).to_string())
}

#[instrument(name = "指标-全部")]
pub async fn all_metrics() -> Result<Metrics, AppErr> {
    let success = METRIC_SUCCESS.load(Ordering::Relaxed);
    let fail = METRIC_FAIL.load(Ordering::Relaxed);
    let request = METRIC_REQUEST.load(Ordering::Relaxed);
    let banned = METRIC_BANNED.load(Ordering::Relaxed);

    Ok(Metrics {
        success,
        fail,
        request,
        banned,
    })
}

#[instrument(name = "指标-SSE")]
pub async fn sse() -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppErr> {
    let stream = stream::repeat_with(|| {
        let success = METRIC_SUCCESS.load(Ordering::Relaxed);
        let fail = METRIC_FAIL.load(Ordering::Relaxed);
        let request = METRIC_REQUEST.load(Ordering::Relaxed);
        let banned = METRIC_BANNED.load(Ordering::Relaxed);
        let metrics = Metrics {
            success,
            fail,
            request,
            banned,
        };
        Event::default()
            .json_data(metrics)
            .unwrap_or_else(|_| Event::default())
    })
    .map(Ok)
    .throttle(Duration::from_millis(METRICS_DELAY));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[instrument(name = "指标-历史", skip(state))]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<model::Metrics>>, AppErr> {
    let mut db = state.db;
    let data = model::Metrics::all()
        .latest_by(model::Metrics::fields().time())
        .limit(15)
        .exec(&mut db)
        .await?;
    Ok(Json(data))
}
