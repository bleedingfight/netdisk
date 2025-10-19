use crate::responses::prelude::*;
use actix_web::{get, web, HttpResponse};
use log::{debug, error, info};
use reqwest;
use std::error::Error;

// TODO 返回用户信息应该加密
#[get("/user_info")]
pub async fn user_info(token: web::Data<AccessToken>) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/user/info", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    let response = client
        .get(&api_url)
        .header("Authorization", &authorization_header)
        .header("Platform", platform.platform())
        .send()
        .await
        .map_err(|e| format!("请求发送失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into())
    } else {
        // 解析响应
        let api_response: UserInfoResponse = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        Ok(HttpResponse::Ok().json(api_response))
    }
}
