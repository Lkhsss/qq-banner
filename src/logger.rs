use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init() {
    // 注册一个全局的日志记录器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("qq_banner=info,axum=info,tower_http=info"));
    let timer = ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f".to_string());
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false) //不打印target
                .with_timer(timer),
        )
        .init();
}
