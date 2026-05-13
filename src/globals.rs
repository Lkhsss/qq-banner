use std::sync::atomic::AtomicU64;

use include_dir::include_dir;

pub const DATA_DIR: &str = "./data";
pub const DB_PATH: &str = "namelist.sqlite";
pub const ADDR: &str = "0.0.0.0";
pub const API_PORT: &str = "6100";
pub const WEBUI_PORT: &str = "6101";
pub const DIST_DIR: &str = "./dist";
pub const EXPIRE_TIME: i64 = 604800;
pub const SALT: &str = "qq-banner";

// 分页
pub const PAGING_DEFAULT: usize = 20;
// 指标的sse推送间隔
pub const METRICS_DELAY: u64 = 3000;
// 数据库刷新时间
pub const DATABASE_FLUSH_DELAY: u64 = 1000;

pub static PROJECT_DIR: include_dir::Dir = include_dir!("./DCM-panel/dist"); //将前端硬编码到项目

pub static METRIC_SUCCESS: AtomicU64 = AtomicU64::new(0);
pub static METRIC_FAIL: AtomicU64 = AtomicU64::new(0);
pub static METRIC_REQUEST: AtomicU64 = AtomicU64::new(0);
pub static METRIC_BANNED: AtomicU64 = AtomicU64::new(0);
