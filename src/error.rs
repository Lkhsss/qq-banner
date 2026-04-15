use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum AppErr {
    #[error("数据库出现错误: {0}")]
    Database(#[from] toasty::Error),
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        let (msg, statuscode) = match self {
            AppErr::Database(_) => (self.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
        };

        (statuscode, msg).into_response()
    }
}
