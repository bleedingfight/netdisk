use crate::io_basic::read_and_write::*;
use crate::netdisk_auth::basic_env::NetDiskEnv;
use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use log::{debug, error, info};
use reqwest;
use std::error::Error;
use std::path::Path;
#[get("/file_lists_query")]
pub async fn file_lists_query(
    query: web::Query<FileListQuery>, // 假设 FileListQuery 包含所有参数
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v2/file/list", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);

    // --- 修正 3: 构建包含所有可选参数的查询参数列表 ---
    let mut query_params = Vec::new();
    query_params.push(("parentFileId", query.parent_file_id.to_string()));
    query_params.push(("limit", query.limit.to_string()));

    // 动态添加可选参数
    if let Some(search_data) = &query.search_data {
        query_params.push(("searchData", search_data.clone()));
    }
    if let Some(search_mode) = query.search_mode {
        query_params.push(("searchMode", search_mode.to_string()));
    }
    if let Some(last_file_id) = query.last_file_id {
        query_params.push(("lastFileId", last_file_id.to_string()));
    }

    // 1. 发送 GET 请求
    // debug!("尝试发送信息:{}", &query_params);
    let response = client
        .get(api_url)
        .query(&query_params) // 使用包含所有参数的 Vec
        .header("Content-Type", "application/json")
        .header("Platform", "open_platform")
        .header("Authorization", &authorization_header)
        .send()
        .await?;

    // 2. 检查 HTTP 状态码
    if !response.status().is_success() {
        // ... (错误处理逻辑不变)
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    }
    debug!("请求成功");

    // --- 修正 1 & 4: 正确解析和返回 ---
    let api_response: FileListResponse = response.json().await?;

    // 返回 Actix 响应
    Ok(HttpResponse::Ok().json(api_response))
}

#[get("/file_query")]
pub async fn file_query(
    query: web::Query<FileQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/file/detail", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    // 关键修复：使用与API匹配的参数名fileID
    let mut query_params = Vec::new();
    query_params.push(("fileID", query.file_id.to_string())); // 这里改为fileID

    debug!("尝试发送信息: {:?}", &query_params);
    let response = client
        .get(api_url)
        .query(&query_params)
        .header("Platform", "open_platform")
        .header("Authorization", &authorization_header)
        .send()
        .await
        .map_err(|e| format!("请求发送失败: {}", e))?;

    // 检查HTTP状态码
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into());
    }

    // 解析响应
    let api_response: FileResponse = response
        .json()
        .await
        .map_err(|e| format!("响应解析失败: {}", e))?;

    Ok(HttpResponse::Ok().json(api_response))
}

#[post("/files_info")]
pub async fn files_info(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/file/infos", platform.platform_domain());

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
        return Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into());
    }

    // 解析响应
    let api_response: FilesInfoResponse = response
        .json()
        .await
        .map_err(|e| format!("响应解析失败: {}", e))?;

    Ok(HttpResponse::Ok().json(api_response))
}

#[post("/mkdir")]
pub async fn mkdir(
    payload: web::Json<EntryItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/upload/v1/file/mkdir",
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
        .map_err(|e| format!("请求发送失败: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into())
    } else {
        // 解析响应
        let api_response: PathInfoResponse = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}
