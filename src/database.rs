use std::{sync::atomic::Ordering, time::Duration};

use axum::{Json, response::IntoResponse};
use qq_banner::{
    DATABASE_FLUSH_DELAY, DAY_FAIL, DAY_REQUEST, DAY_SUCCESS, METRIC_BANNED, METRIC_FAIL,
    METRIC_REQUEST, METRIC_SUCCESS,
    model::{self, Manager, Reporter, User},
};
use serde::Serialize;
use sled::IVec;
use toasty;
use tokio::time::sleep;

use crate::{
    AppState,
    error::AppErr,
    handler::{UserStatusBack, now_unix_secs},
};

pub trait Banner {
    fn is_ban_expired(user: &User, now: u64) -> bool;
    async fn ban<U: AsRef<User>>(&mut self, new_user: U) -> Result<UserStatusBack, AppErr>;
    async fn get_permisson(&mut self, id: &str) -> Result<i16, AppErr>;
    async fn report(&mut self, id: u64) -> Result<u64, AppErr>;
    async fn get_report(&mut self, id: u64) -> Result<u64, AppErr>;
    async fn clean_report(&mut self, id: u64) -> Result<u64, AppErr>;
    async fn get_metric_day(&mut self) -> Result<model::Metrics, AppErr>;
    async fn set_metric_day_request(&mut self, n: u64) -> Result<u64, AppErr>;
    async fn set_metric_day_success(&mut self, n: u64) -> Result<u64, AppErr>;
    async fn set_metric_day_fail(&mut self, n: u64) -> Result<u64, AppErr>;
}

impl Banner for toasty::Db {
    fn is_ban_expired(user: &User, now: u64) -> bool {
        user.duration != 0 && now >= user.time.saturating_add(user.duration)
    }
    /// # 封禁用户，并计数器+1
    /// 分离出来的业务函数
    async fn ban<U: AsRef<User>>(&mut self, new_user: U) -> Result<UserStatusBack, AppErr> {
        let new_user = new_user.as_ref();
        let users = User::filter(User::fields().id().eq(new_user.id))
            .first()
            .exec(self)
            .await?;
        match users {
            Some(mut u) => {
                u.update()
                    .operator(&new_user.operator)
                    .duration(new_user.duration)
                    .time(new_user.time)
                    .exec(self)
                    .await?;
                Ok(UserStatusBack::banned(new_user))
            }
            None => {
                let user = toasty::create!(User {
                    id: new_user.id,
                    time: new_user.time,
                    duration: new_user.duration,
                    operator: &new_user.operator,
                })
                .exec(self)
                .await?;
                METRIC_BANNED.fetch_add(1, Ordering::Relaxed);
                Ok(UserStatusBack::banned(user))
            }
        }
    }

    /// # 获取数据库中权限
    /// 如未在库中权限默认为-1
    async fn get_permisson(&mut self, id: &str) -> Result<i16, AppErr> {
        let user = Manager::filter(Manager::fields().name().eq(id))
            .first()
            .exec(self)
            .await?;
        match user {
            Some(u) => Ok(u.permission),
            None => Ok(-1),
        }
    }
    /// 举报用户
    /// report字段+1
    async fn report(&mut self, id: u64) -> Result<u64, AppErr> {
        let name = id.to_string();
        let reporter = Reporter::filter_by_name(name.clone())
            .first()
            .exec(self)
            .await?;
        let count = match reporter {
            Some(mut r) => {
                let count = r.count.saturating_add(1);
                r.update().count(count).exec(self).await?;
                count
            }
            None => {
                toasty::create!(Reporter { name, count: 1 })
                    .exec(self)
                    .await?;
                1
            }
        };
        Ok(count)
    }
    async fn get_report(&mut self, id: u64) -> Result<u64, AppErr> {
        let name = id.to_string();
        let reporter = Reporter::filter_by_name(name.clone())
            .first()
            .exec(self)
            .await?;
        let count = match reporter {
            Some(r) => r.count,
            None => 0,
        };
        Ok(count)
    }

    async fn clean_report(&mut self, id: u64) -> Result<u64, AppErr> {
        let reporter = Reporter::filter_by_name(id.to_string())
            .first()
            .exec(self)
            .await?;
        match reporter {
            Some(mut r) => {
                r.update().count(0).exec(self).await?;
            }
            None => (),
        };
        Ok(0)
    }
    /// 获取数据库中的metric数据
    async fn get_metric_day(&mut self) -> Result<model::Metrics, AppErr> {
        Ok(model::Metrics::all()
            .latest_by(model::Metrics::fields().request())
            .one()
            .exec(self)
            .await?)
    }

    async fn set_metric_day_request(&mut self, n: u64) -> Result<u64, AppErr> {
        model::Metrics::all()
            .latest_by(model::Metrics::fields().request())
            .update()
            .request(n)
            .exec(self)
            .await?;
        Ok(n)
    }

    async fn set_metric_day_success(&mut self, n: u64) -> Result<u64, AppErr> {
        model::Metrics::all()
            .latest_by(model::Metrics::fields().request())
            .update()
            .success(n)
            .exec(self)
            .await?;
        Ok(n)
    }

    async fn set_metric_day_fail(&mut self, n: u64) -> Result<u64, AppErr> {
        model::Metrics::all()
            .latest_by(model::Metrics::fields().request())
            .update()
            .fail(n)
            .exec(self)
            .await?;
        Ok(n)
    }
}

/// # 从数据库获取被封禁的人数
pub async fn banned_user_count(db: &mut toasty::Db) -> Result<u64, AppErr> {
    let users = User::all().exec(db).await?;
    let now = now_unix_secs();
    let mut count = 0;

    for user in users {
        if toasty::Db::is_ban_expired(&user, now) {
            user.delete().exec(db).await?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

/// 指标聚合
#[derive(Debug, Serialize)]
pub struct Metrics {
    pub success: u64,
    pub fail: u64,
    pub request: u64,
    pub banned: u64,
}

/// # 获取指定指标
pub async fn get_metric(name: &str, db: &sled::Db) -> Result<u64, AppErr> {
    let data = db
        .get(name.as_bytes())?
        .unwrap_or(IVec::from(0_u64.to_be_bytes().to_vec()));
    Ok(u64::from_be_bytes(data.as_ref().try_into()?))
}

impl IntoResponse for Metrics {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

/// # 同步数据库和原子自增
pub async fn sync_metrics(sled_db: &sled::Db, toasty_db: &mut toasty::Db) -> Result<(), AppErr> {
    // 总计数器
    let success = get_metric("counter:success", sled_db).await.unwrap_or(0);
    let fail = get_metric("counter:fail", sled_db).await.unwrap_or(0);
    let request = get_metric("counter:request", sled_db).await.unwrap_or(0);
    // 日计数器

    METRIC_SUCCESS.store(success, Ordering::Relaxed);
    METRIC_FAIL.store(fail, Ordering::Relaxed);
    METRIC_REQUEST.store(request, Ordering::Relaxed);

    // 日指标读取
    let day_metric = model::Metrics::all()
        .filter_by_time(get_metric_time())
        .first()
        .exec(toasty_db)
        .await?;

    match day_metric {
        Some(d) => {
            DAY_REQUEST.store(d.request, Ordering::Relaxed);
            DAY_SUCCESS.store(d.success, Ordering::Relaxed);
            DAY_FAIL.store(d.fail, Ordering::Relaxed);
        }
        None => {
            // 初始化表
            toasty::create!(model::Metrics {
                time: get_metric_time(),
                request: 0,
                success: 0,
                fail: 0,
            })
            .exec(toasty_db)
            .await?;
        }
    }

    Ok(())
}

pub fn start_persist_task(state: AppState) {
    tokio::spawn(async move {
        println!("✅ 后台计数持久化任务已启动");
        let metrics = state.metrics;
        let mut toasty_db = state.db;
        loop {
            sleep(Duration::from_millis(DATABASE_FLUSH_DELAY)).await;

            // 1. 读取内存计数
            let request = METRIC_REQUEST.load(Ordering::Relaxed);
            let success = METRIC_SUCCESS.load(Ordering::Relaxed);
            let fail = METRIC_FAIL.load(Ordering::Relaxed);

            let day_request = DAY_REQUEST.load(Ordering::Relaxed);
            let day_success = DAY_SUCCESS.load(Ordering::Relaxed);
            let day_fail = DAY_FAIL.load(Ordering::Relaxed);

            // 2. 写入 sled
            let _ = &metrics
                .update_and_fetch(b"counter:success", |_| Some(success.to_be_bytes().to_vec()));
            let _ =
                &metrics.update_and_fetch(b"counter:fail", |_| Some(fail.to_be_bytes().to_vec()));
            let _ = &metrics
                .update_and_fetch(b"counter:request", |_| Some(request.to_be_bytes().to_vec()));

            //写入sqlite
            let _ = toasty_db.set_metric_day_request(day_request).await;
            let _ = toasty_db.set_metric_day_success(day_success).await;
            let _ = toasty_db.set_metric_day_fail(day_fail).await;
        }
    });
}

pub fn get_metric_time() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
