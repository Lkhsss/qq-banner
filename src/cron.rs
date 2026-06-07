use anyhow::Result;
use toasty::Db;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::database::get_metric_time;

/// 启动每日凌晨创建 Metrics 新记录的定时任务。
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
            })
        }))
        .build()?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}
