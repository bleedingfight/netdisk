use crate::netdisk_api::api_client::ApiClient;
use crate::responses::prelude::*;
use actix_web::{web, HttpResponse};
use log::debug;
use reqwest;
use std::error::Error;

pub async fn share_create(
    payload: web::Json<ShareItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/share/create", platform.platform_domain());

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
        let api_response: SharedDataResponse = response
            .json()
            .await
            .map_err(|e| format!("响应解析失败: {}", e))?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}

pub async fn share_list(
    query: web::Query<ShareQuery>, // 假设 FileListQuery 包含所有参数
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "/api/v1/share/list"
    );

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);

    let mut query_params = Vec::new();
    query_params.push(("limit", query.limit.to_string()));

    if let Some(last_share_id) = query.last_share_id {
        query_params.push(("lastFileId", last_share_id.to_string()));
    }

    // 1. 发送 GET 请求
    debug!("尝试发送信息:{:?}", &query);

    let response = client
        .get(api_url)
        .query(&query_params) // 使用包含所有参数的 Vec
        .header("Content-Type", "application/json")
        .header("Platform", platform.platform())
        .header("Authorization", &authorization_header)
        .send()
        .await?;

    // 2. 检查 HTTP 状态码
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    } else {
        let api_response: SharedListDataResponse = response.json().await?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}

// #[get("/list")]
pub async fn share_list_info(
    payload: web::Json<ShareLinkItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "api/v1/share/list/info"
    );

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);

    // 1. 发送 GET 请求
    debug!("尝试发送信息:{:?}", &payload);
    let response = client
        .put(api_url)
        .header("Platform", platform.platform())
        // 设置 Authorization Header
        .header("Authorization", authorization_header)
        // 使用 .json() 方法自动设置 Content-Type: application/json 并序列化 payload
        .json(&payload)
        .send()
        .await?;

    // 2. 检查 HTTP 状态码
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    } else {
        let api_response: ApiResponse<()> = response.json().await?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}

/// # 创建付费分享链接
pub async fn pay_link(
    payload: web::Json<PayLinkItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "api/v1/share/content-payment/create"
    );

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);

    debug!("尝试发送信息:{:?}", &payload);
    let response = client
        .post(api_url)
        .header("Platform", platform.platform())
        .header("Authorization", authorization_header)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    } else {
        let api_response: SharedDataResponse = response.json().await?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}
/// #获取付费分享链接列表
pub async fn payment_list(
    query: web::Query<ShareQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let share_query_data: ShareQuery = query.into_inner();
    debug!("尝试发送信息:{:?}", &share_query_data);
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "api/v1/share/payment/list"
    );

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);
    let response = client
        .get(api_url)
        .header("Content-Type", "application/json") // GET 请求中这个头可选，但无害
        .header("Platform", platform.platform())
        .header("Authorization", &authorization_header)
        .query(&share_query_data) // ✅ 正确：将 ShareQuery 结构体传递给 .query()
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    } else {
        let api_response: PayShareDataResponse = response.json().await?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}

/// # 修改付费分享文件信息
pub async fn change_share_list_info(
    payload: web::Json<ShareLinkItem>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "api/v1/share/list/payment/info"
    );

    let authorization_header = format!("Bearer {}", token.get_ref().access_token);

    // 1. 发送 GET 请求
    debug!("尝试发送信息:{:?}", &payload);
    let response = client
        .put(api_url)
        .header("Platform", platform.platform())
        .header("Authorization", authorization_header)
        .json(&payload)
        .send()
        .await?;

    // 2. 检查 HTTP 状态码
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 请求失败，HTTP 状态码: {}，响应: {}", status, body).into());
    } else {
        let api_response: ApiResponse<()> = response.json().await?;
        Ok(HttpResponse::Ok().json(api_response))
    }
}

// ============================================================================
// 可测试版本的 API 函数
// ============================================================================

/// 获取分享列表 - 可测试版本
///
/// 对应 curl: curl -X GET http://127.0.0.1:8080/share/list?limit=10
pub async fn share_list_v2(
    query: web::Query<ShareQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    let mut query_params = Vec::new();
    query_params.push(("limit", query.limit.to_string()));

    if let Some(last_share_id) = query.last_share_id {
        query_params.push(("lastFileId", last_share_id.to_string()));
    }

    debug!("share_list_v2: 获取分享列表");

    let response = client.http_client()
        .get(client.url("/api/v1/share/list"))
        .query(&query_params)
        .header("Content-Type", "application/json")
        .header("Platform", client.platform())
        .header("Authorization", &authorization)
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

    let api_response: SharedListDataResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 创建分享链接 - 可测试版本
///
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"shareFileIdList":[123], "sharePwd":"1234", "expiredTime":"2099-12-31 23:59:59"}' \
///   http://127.0.0.1:8080/share/create
pub async fn share_create_v2(
    payload: web::Json<ShareItem>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("share_create_v2: 创建分享链接 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/api/v1/share/create"))
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

    let api_response: SharedDataResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

pub fn share_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/share") // 所有路由都以 /share 为前缀
            .route("/create", web::post().to(share_create))
            .route("/list", web::get().to(share_list))
            .route("/list/info", web::put().to(share_list_info))
            .route("/content-payment/creat", web::put().to(pay_link))
            .route("/list/payment/info", web::put().to(change_share_list_info))
            .route("/payment/list", web::get().to(payment_list)),
    );
}
