use crate::netdisk_api::api_client::ApiClient;
use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use log::{debug, error, info};
use reqwest;
use std::error::Error;

#[post("/trash")]
pub async fn trash(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/file/trash", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    debug!("尝试发送信息: {:?}", &payload);

    let response = client
        .post(&api_url)
        .header("Authorization", &authorization_header)
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("请求发送失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into())
    } else {
        // 解析响应
        let api_response: ApiResponse<()> = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}

/// 移动文件到回收站 - 可测试版本
///
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"fileIds":[123,456]}' http://127.0.0.1:8080/trash
pub async fn trash_v2(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("trash_v2: 移动文件到回收站 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/api/v1/file/trash"))
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

#[post("/delete")]
pub async fn delete(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/file/delete", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    debug!("尝试发送信息: {:?}", &payload);

    let response = client
        .post(&api_url)
        .header("Authorization", &authorization_header)
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("请求发送失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into())
    } else {
        // 解析响应
        let api_response: ApiResponse<()> = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}

/// 永久删除文件 - 可测试版本
///
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"fileIds":[123,456]}' http://127.0.0.1:8080/delete
pub async fn delete_v2(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("delete_v2: 永久删除文件 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/api/v1/file/delete"))
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
