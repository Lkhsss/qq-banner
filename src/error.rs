use std::array::TryFromSliceError;

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum AppErr {
    #[error("数据库出现错误: {0}")]
    Database(#[from] toasty::Error),
    #[error("密钥错误")]
    BadPassword,
    #[error("io错误：{0}")]
    Io(#[from] std::io::Error),
    #[error("上游接口请求失败: {0}")]
    Upstream(#[from] reqwest::Error),
    #[error("创建token失败: {0}")]
    CreateTokenErr(#[from] jsonwebtoken::errors::Error),
    #[error("Sled数据库出现错误: {0}")]
    SledErr(#[from] sled::Error),
    #[error("数据转换出错: {0}")]
    Conversion(#[from] TryFromSliceError),
    #[error("权限不足")]
    PermissonDenied,
    #[error("用户不存在")]
    UserNotFound,
    #[error("管理员已存在")]
    ManagerExists,
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        let (msg, statuscode) = match self {
            AppErr::Database(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            AppErr::BadPassword => (self.to_string(), StatusCode::UNAUTHORIZED),
            AppErr::Io(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            AppErr::Upstream(_) => (self.to_string(), StatusCode::BAD_GATEWAY),
            AppErr::CreateTokenErr(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            AppErr::SledErr(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            AppErr::Conversion(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
            AppErr::PermissonDenied => (self.to_string(), StatusCode::FORBIDDEN),
            AppErr::UserNotFound => (self.to_string(), StatusCode::UNAUTHORIZED),
            AppErr::ManagerExists => (self.to_string(), StatusCode::CONFLICT),
        };

        (statuscode, msg).into_response()
    }
}
