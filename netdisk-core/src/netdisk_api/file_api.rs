use crate::netdisk_api::api_client::ApiClient;
use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse};
use log::{debug, error, info};
use reqwest;
use std::error::Error;
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
        .header("Platform", platform.platform())
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
    let api_response: FileListResponse = response.json().await?;

    // 返回 Actix 响应
    Ok(HttpResponse::Ok().json(api_response))
}

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
        .header("Platform", platform.platform())
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

pub async fn download(
    query: web::Query<FileQuery>,
    token: web::Data<AccessToken>,
) -> Result<HttpResponse, actix_web::Error> {
    // <- 注意返回类型
    let file_query_data: FileQuery = query.into_inner();
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!(
        "https://{}/{}",
        platform.platform_domain(),
        "api/v1/file/download_info"
    );

    let authorization_header = format!("Bearer {}", token.access_token);

    debug!("尝试发送信息: {:?}", &file_query_data);

    let response = client
        .get(api_url)
        .query(&file_query_data)
        .header("Content-Type", "application/json")
        .header("Platform", platform.platform())
        .header("Authorization", &authorization_header)
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?; // <- 转 actix_web::Error

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        Err(actix_web::error::ErrorInternalServerError(format!(
            "API请求失败，状态码: {}，响应: {}",
            status, body
        )))
    } else {
        let api_response: DownloadUrlResponse = response
            .json()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        debug!("响应内容: {:?}", &api_response);
        Ok(HttpResponse::Ok().json(api_response))
    }
}

pub fn file_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/file") // 所有路由都以 /share 为前缀
            .route("/download", web::get().to(download))
            .route("/mkdir", web::post().to(mkdir))
            .route("/file_lists_query", web::get().to(file_lists_query))
            .route("/file_query", web::get().to(file_query))
            .route("/files_info", web::post().to(files_info)),
    );
}

// ============================================================================
// 可测试版本的 API 函数 (使用依赖注入的 ApiClient)
// ============================================================================

/// 文件查询 - 可测试版本
/// 
/// 通过 web::Data<ApiClient> 注入 API 客户端，使其可以在测试中替换为 mock server
/// 
/// # Example
/// ```ignore
/// // 在测试中使用
/// let client = ApiClient::with_config(ApiClientConfig::for_test(mock_server.uri()));
/// let app = test::init_service(
///     App::new()
///         .app_data(web::Data::new(client))
///         .app_data(web::Data::new(token))
///         .route("/file_query", web::get().to(file_query_v2))
/// ).await;
/// ```
pub async fn file_query_v2(
    query: web::Query<FileQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);
    
    // 构建查询参数
    #[derive(serde::Serialize)]
    struct QueryParams {
        #[serde(rename = "fileID")]
        file_id: i64,
    }
    
    let params = QueryParams { file_id: query.file_id };
    
    debug!("file_query_v2: 查询文件 {:?}", &params.file_id);
    
    let response = client.http_client()
        .get(client.url("/api/v1/file/detail"))
        .query(&params)
        .header("Platform", client.platform())
        .header("Authorization", &authorization)
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "API请求失败，状态码: {}，响应: {}",
            status, body
        )));
    }

    let api_response: FileResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 文件列表查询 - 可测试版本
pub async fn file_lists_query_v2(
    query: web::Query<FileListQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    let mut query_params = Vec::new();
    query_params.push(("parentFileId", query.parent_file_id.to_string()));
    query_params.push(("limit", query.limit.to_string()));

    if let Some(search_data) = &query.search_data {
        query_params.push(("searchData", search_data.clone()));
    }
    if let Some(search_mode) = query.search_mode {
        query_params.push(("searchMode", search_mode.to_string()));
    }
    if let Some(last_file_id) = query.last_file_id {
        query_params.push(("lastFileId", last_file_id.to_string()));
    }

    let response = client.http_client()
        .get(client.url("/api/v2/file/list"))
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

    let api_response: FileListResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 批量文件信息查询 - 可测试版本
pub async fn files_info_v2(
    payload: web::Json<FilesQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("files_info_v2: 查询文件 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/api/v1/file/infos"))
        .header("Authorization", &authorization)
        .header("Platform", client.platform())
        .json(&payload.into_inner())
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(actix_web::error::ErrorInternalServerError(format!(
            "API请求失败，状态码: {}，响应: {}",
            status, body
        )));
    }

    let api_response: FilesInfoResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 创建目录 - 可测试版本
///
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"name":"新文件夹", "parentID":0}' http://127.0.0.1:8080/file/mkdir
pub async fn mkdir_v2(
    payload: web::Json<EntryItem>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    debug!("mkdir_v2: 创建目录 {:?}", &payload);

    let response = client.http_client()
        .post(client.url("/upload/v1/file/mkdir"))
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

    let api_response: PathInfoResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 获取下载链接 - 可测试版本
///
/// 对应 curl: curl -X GET http://127.0.0.1:8080/file/download?fileId=xxx
pub async fn download_v2(
    query: web::Query<FileQuery>,
    token: web::Data<AccessToken>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let authorization = format!("Bearer {}", token.access_token);

    #[derive(serde::Serialize)]
    struct QueryParams {
        #[serde(rename = "fileID")]
        file_id: i64,
    }

    let params = QueryParams { file_id: query.file_id };

    debug!("download_v2: 获取下载链接 {:?}", &params.file_id);

    let response = client.http_client()
        .get(client.url("/api/v1/file/download_info"))
        .query(&params)
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

    let api_response: DownloadUrlResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(api_response))
}

/// 可测试版本的路由配置
pub fn file_config_v2(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/file")
            .route("/download", web::get().to(download_v2))
            .route("/mkdir", web::post().to(mkdir_v2))
            .route("/file_lists_query", web::get().to(file_lists_query_v2))
            .route("/file_query", web::get().to(file_query_v2))
            .route("/files_info", web::post().to(files_info_v2)),
    );
}
