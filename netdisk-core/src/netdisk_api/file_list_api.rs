use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use log::{debug, error, info};
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

#[get("/file_search")]
pub async fn file_search(
    query: web::Query<FileSearchItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v2/file/list", platform.platform_domain());

    let authorization_header = format!("Bearer {}", token.access_token);

    let mut query_params = Vec::new();
    query_params.push(("parentFileId", query.parent_file_id.to_string()));
    query_params.push(("limit", query.limit.to_string()));

    // 动态添加可选参数
    if let Some(search_data) = &query.search_data {
        query_params.push(("searchData", search_data.clone()));
    }
    if let Some(search_mode) = &query.search_mode {
        query_params.push(("searchMode", search_mode.to_string()));
    }
    if let Some(last_file_id) = &query.last_file_id {
        query_params.push(("lastFileId", last_file_id.to_string()));
    }

    let response = client
        .get(api_url)
        .query(&query_params) // 使用包含所有参数的 Vec
        .header("Content-Type", "application/json")
        .header("Platform", "open_platform")
        .header("Authorization", &authorization_header)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(format!("API请求失败，状态码: {}，响应: {}", status, body).into())
    } else {
        // 解析响应
        let api_response: FileSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}
