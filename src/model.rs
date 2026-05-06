use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, toasty::Model, Serialize)]
pub struct User {
    #[key]
    pub id: u64,
    pub time: u64,
    pub duration: u64,
    pub operator:String
}

impl IntoResponse for User {
    fn into_response(self) -> axum::response::Response {
        Json(json!(self)).into_response()
    }
}

#[derive(Debug, toasty::Model, Serialize, Deserialize)]
#[table = "manager"]
pub struct Manager {
    #[key]
    pub name: String,
    pub password: String,
    #[serde(skip_deserializing)]
    pub permission: i16,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i16)]
pub enum Permission {
    SuperAdmin = 2,
    Admin = 1,
    User = 0,
    None = -1,
}

impl From<Permission> for i16 {
    fn from(value: Permission) -> Self {
        value as i16
    }
}

impl From<i16> for Permission {
    fn from(value: i16) -> Self {
        match value {
            2 => Self::SuperAdmin,
            1 => Self::Admin,
            0 => Self::User,
            _ => Self::None,
        }
    }
}

impl Manager {
    pub fn permission_enum(&self) -> Permission {
        self.permission.into()
    }

    pub fn set_permission(&mut self, permission: Permission) {
        self.permission = permission.into();
    }
}
