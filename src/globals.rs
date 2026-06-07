use std::sync::atomic::AtomicU64;

pub const ADMIN_USER: &str = "admin";
pub const DATA_DIR: &str = "./data";
pub const DB_PATH: &str = "namelist.sqlite";
pub const ADDR: &str = "0.0.0.0";
pub const API_PORT: u16 = 6100;
pub const WEBUI_PORT: u16 = 6101;
pub const NAPCAT_ADDR: &str = "http://111.228.4.19";
pub const NAPCAT_PORT: u16 = 8000;
pub const NAPCAT_TOKEN: &str = "FTjQR2sso7LZlaql";

pub const EXPIRE_TIME: i64 = 604800;
pub const SALT: &str = "qq-banner";

// 分页
pub const PAGING_DEFAULT: usize = 20;
// 指标的sse推送间隔
pub const METRICS_DELAY: u64 = 3000;
// 数据库刷新时间
pub const DATABASE_FLUSH_DELAY: u64 = 2000;

// pub static PROJECT_DIR: include_dir::Dir = include_dir!("./DCM-panel/dist"); //将前端硬编码到项目

pub static METRIC_SUCCESS: AtomicU64 = AtomicU64::new(0);
pub static METRIC_FAIL: AtomicU64 = AtomicU64::new(0);
pub static METRIC_REQUEST: AtomicU64 = AtomicU64::new(0);
pub static METRIC_BANNED: AtomicU64 = AtomicU64::new(0);

pub static DAY_SUCCESS: AtomicU64 = AtomicU64::new(0);
pub static DAY_FAIL: AtomicU64 = AtomicU64::new(0);
pub static DAY_REQUEST: AtomicU64 = AtomicU64::new(0);
