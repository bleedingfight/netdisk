use crate::responses::prelude::*;
use actix_web::{self, get, post, web, HttpResponse, HttpServer, Responder};
use log::{debug, error, info};
use reqwest;
use std::error::Error;

#[post("/file/upload")]
pub async fn file_upload(
    payload: web::Json<UploadFileItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, actix_web::Error> {
    // **注意：理想情况下，client 和 platform 应该通过 web::Data 传入**
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();

    let api_url = format!(
        "https://{}/upload/v2/file/create",
        platform.platform_domain()
    );

    let authorization_header = format!("Bearer {}", token.access_token);

    debug!("尝试发送信息: {:?}", &payload);
    let response = client
        .post(&api_url)
        .header("Authorization", &authorization_header)
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("请求发送失败: {}", e)))?;

    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|e| format!("无法读取失败响应体: {}", e)); // 确保总是有一个字符串

        Err(actix_web::error::ErrorInternalServerError(format!(
            "API请求失败，状态码: {}，响应: {}",
            status, body
        )))
    } else {
        let api_response: UploadFileResponse = response.json().await.map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("响应解析失败: {}", e))
        })?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}
