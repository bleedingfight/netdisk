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
// pub fn move_config(cfg: &mut web::ServiceConfig) {
//     println!("✅ move_config 被调用，注册 /file/move");

//     cfg.service(web::scope("/file").route("/move", web::post().to(move_file)));
// }
