use crate::netdisk_api::api_client::ApiClient;
use crate::responses::prelude::*;
use actix_web::{self, error, post, web, HttpResponse};
use log::debug;
use reqwest;
#[actix_web::route("/file/move", method = "POST")]
pub async fn move_file(
    payload: web::Json<FileMoveInfo>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("目标 API URL = ");

    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/file/move", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    debug!("尝试发送信息: {:?}", &payload);

    let response = client
        .post(&api_url)
        .header("Authorization", &authorization_header)
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            let error_message = format!("请求发送失败: {}", e);
            error::ErrorInternalServerError(error_message)
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let error_message = format!("API请求失败，状态码: {}，响应: {}", status, body);
        return Err(error::ErrorInternalServerError(error_message));
    } else {
        let api_response: ApiResponse<()> = response.json().await.map_err(|e| {
            let error_message = format!("响应解析失败: {}", e);
            error::ErrorInternalServerError(error_message)
        })?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}

/// 移动文件 - 可测试版本
///
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"fileIDs":[123,456], "toParentFileID":789}' http://127.0.0.1:8080/file/move
pub async fn move_file_v2(
    payload: web::Json<FileMoveInfo>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("move_file_v2: 移动文件 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/api/v1/file/move"))
        .header("Authorization", &authorization)
        .header("Platform", client.platform())
        .json(&payload.into_inner())
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "API 请求失败，HTTP 状态码: {}，响应: {}",
            status, body
        )));
    }

    let api_response: ApiResponse<()> = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}
// pub fn move_config(cfg: &mut web::ServiceConfig) {
//     println!("✅ move_config 被调用，注册 /file/move");

//     cfg.service(web::scope("/file").route("/move", web::post().to(move_file)));
// }
