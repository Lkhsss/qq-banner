use std::sync::atomic::Ordering;

use anyhow::Result;
use toasty::Db;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::database::{Banner, get_metric_time};
use qq_banner::globals::{DAY_FAIL, DAY_REQUEST, DAY_SUCCESS};

/// 启动每日凌晨创建 Metrics 新记录的定时任务。
///
/// 每天 0 点：
/// 1. 将当日内存计数器写入数据库
/// 2. 创建新一天的 Metrics 记录
/// 3. 清空内存计数器
pub async fn start_daily_metrics_job(db: Db) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;
    let local_tz = chrono::offset::Local::now().timezone();

    let job = JobBuilder::new()
        .with_timezone(local_tz)
        .with_cron_job_type()
        .with_schedule("0 0 0 * * *")? // 凌晨 0 点执行
        .with_run_async(Box::new(move |_uuid, _l| {
            let mut db = db.clone();
            Box::pin(async move {
                // 1. 读取当日计数器
                let day_request = DAY_REQUEST.load(Ordering::Relaxed);
                let day_success = DAY_SUCCESS.load(Ordering::Relaxed);
                let day_fail = DAY_FAIL.load(Ordering::Relaxed);

                // 2. 将当日计数器写入数据库，然后创建新一天的记录
                if let Err(e) = db.set_metric_day_request(day_request).await {
                    eprintln!("cron: flush day_request failed: {e}");
                }
                if let Err(e) = db.set_metric_day_success(day_success).await {
                    eprintln!("cron: flush day_success failed: {e}");
                }
                if let Err(e) = db.set_metric_day_fail(day_fail).await {
                    eprintln!("cron: flush day_fail failed: {e}");
                }

                if let Err(e) = toasty::create!(qq_banner::model::Metrics {
                    time: get_metric_time(),
                    request: 0,
                    success: 0,
                    fail: 0,
                })
                .exec(&mut db)
                .await
                {
                    eprintln!("cron job failed: {e}");
                }

                // 3. 清空内存计数器
                DAY_REQUEST.store(0, Ordering::Relaxed);
                DAY_SUCCESS.store(0, Ordering::Relaxed);
                DAY_FAIL.store(0, Ordering::Relaxed);
            })
        }))
        .build()?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}
