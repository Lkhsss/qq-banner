use std::{sync::atomic::Ordering, time::Duration};

use axum::{Json, response::IntoResponse};
use qq_banner::{
    DATABASE_FLUSH_DELAY, METRIC_BANNED, METRIC_FAIL, METRIC_REQUEST, METRIC_SUCCESS,
    model::{Manager, Reporter, User},
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
pub async fn sync_metrics(db: &sled::Db) {
    let success = get_metric("counter:success", db).await.unwrap_or(0);
    let fail = get_metric("counter:fail", db).await.unwrap_or(0);
    let request = get_metric("counter:request", db).await.unwrap_or(0);
    METRIC_SUCCESS.store(success, Ordering::Relaxed);
    METRIC_FAIL.store(fail, Ordering::Relaxed);
    METRIC_REQUEST.store(request, Ordering::Relaxed);
}

pub fn start_persist_task(state: AppState) {
    tokio::spawn(async move {
        println!("✅ 后台计数持久化任务已启动");
        let metrics = state.metrics;
        loop {
            sleep(Duration::from_millis(DATABASE_FLUSH_DELAY)).await;

            // 1. 读取内存计数
            let success = METRIC_SUCCESS.load(Ordering::Relaxed);
            let fail = METRIC_FAIL.load(Ordering::Relaxed);
            let request = METRIC_REQUEST.load(Ordering::Relaxed);
            // println!("总数：{} 写入数据库", request);
            //TODO 加入日志系统

            // 2. 写入 sled
            let _ = &metrics
                .update_and_fetch(b"counter:success", |_| Some(success.to_be_bytes().to_vec()));
            let _ =
                &metrics.update_and_fetch(b"counter:fail", |_| Some(fail.to_be_bytes().to_vec()));
            let _ = &metrics
                .update_and_fetch(b"counter:request", |_| Some(request.to_be_bytes().to_vec()));
        }
    });
}
