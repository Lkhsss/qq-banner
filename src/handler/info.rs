use crate::error::AppErr;
use axum::{Form, Json};
use qq_banner::{NAPCAT_ADDR, NAPCAT_PORT, NAPCAT_TOKEN};

use serde::{Deserialize, Serialize};

//TODO用宏重用
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    #[serde(rename(serialize = "id"))]
    user_id: usize,
    nick: String,
    phoneNum: String,
    qqLevel: usize,
    regTime: usize,
}

pub async fn get_stranger_info(Form(form): Form<RequestBuilder>) -> Result<Json<Info>, AppErr> {
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
