use crate::error::AppErr;
use crate::extracter::{AuthManager, UserOrAbove};
use axum::{Json, extract::Query};
use cached::cached;
use qq_banner::{NAPCAT_ADDR, NAPCAT_PORT, NAPCAT_TOKEN};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestBuilder {
    #[serde(rename(deserialize = "id"))]
    user_id: String,
    #[serde(default)]
    no_cache: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponse {
    status: String,
    data: Info,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    #[serde(rename(serialize = "id"))]
    user_id: Option<usize>,
    nick: Option<String>,
    phoneNum: Option<String>,
    qqLevel: Option<usize>,
}
/// 获取qq信息
#[cached(
    size = 50000,
    ttl = 86400,
    result,
    key = "String",
    convert = r#"{ form.user_id.clone() }"#
)]
pub async fn get_stranger_info(
    Query(form): Query<RequestBuilder>,
    _auth: AuthManager<UserOrAbove>,
) -> Result<Json<Info>, AppErr> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{NAPCAT_ADDR}:{NAPCAT_PORT}/get_stranger_info"))
        .json(&form)
        .bearer_auth(NAPCAT_TOKEN)
        .send()
        .await?;

    let data: InfoResponse = response.json().await?;

    Ok(Json(data.data))
}
