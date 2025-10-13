use crate::io_basic::read_and_write::*;
use crate::netdisk_auth::basic_env::NetDiskEnv;
use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use log::{debug, error, info};
use reqwest;
use std::error::Error;
use std::path::Path;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello 123Pan!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// #[get("/list")]
// pub async fn list(info:web:: )
// #[post("/create")]
// pub async fn create(info: web::Json<Config>) -> Result<CreateFileResponse, Box<dyn Error>> {}

// fn save_request_to_file<T>(path: &PathBuf, data: &T) -> Result<(), io::Error> {
//     // 1. 序列化为美观的 TOML 格式字符串
//     let toml_content = toml::to_string_pretty(data)
//         .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("TOML 序列化失败: {}", e)))?;

//     // 2. 构造唯一的文件名 (后缀改为 .toml)
//     let timestamp = Local::now().format("%Y%m%d_%H%M%S");
//     let full_path = path.join("config.toml");

//     // 3. 阻塞地写入文件
//     fs::write(&full_path, toml_content)?;

//     Ok(())
// }

// /// 路由 Handler 函数
// async fn process_request_handler(
//     // 提取全局配置目录 PathBuf (必须有此参数才能获取全局变量)
//     config_dir_data: web::Data<PathBuf>,

//     // 提取 JSON 请求体
//     json_data: web::Json<AuthConfig>,
// ) -> Result<HttpResponse, io::Error> {
//     // 1. 获取请求数据的所有权
//     let request_data = json_data.into_inner();

//     // 2. 条件检查
//     if request_data.cache {
//         // --- 写入逻辑只在 cache=true 时执行 ---

//         // 克隆 PathBuf 的引用（智能指针内部的 PathBuf），供新的线程使用
//         let config_path = config_dir_data.get_ref().clone();

//         // 克隆请求数据，供新的线程使用（因为 request_data 在 if 块后还会被用到）
//         let data_to_save = request_data.clone();

//         // 使用 web::block 将文件写入操作转移到阻塞线程池
//         let save_result =
//             web::block(move || save_request_to_file(&config_path, &data_to_save)).await;

//         // 检查写入结果 (可以选择忽略或返回 500 错误)
//         match save_result {
//             Ok(Ok(())) => {
//                 // 写入成功，打印信息，继续返回响应
//                 println!("[INFO] 缓存写入成功。");
//             }
//             Ok(Err(e)) => {
//                 // 文件 I/O 错误
//                 eprintln!("[ERROR] 文件 I/O 错误，缓存失败: {}", e);
//                 // ⚠️ 注意：这里我们选择继续返回数据，不中断用户请求，但可以记录错误。
//             }
//             Err(e) => {
//                 // web::block 线程池执行错误
//                 eprintln!("[FATAL] 阻塞任务执行失败: {}", e);
//             }
//         }

//         // --- 写入逻辑结束 ---
//     }

//     // 3. 无论是否写入文件，都将接收到的数据返回给用户
//     Ok(HttpResponse::Ok().json(request_data))
// }
pub async fn access_token(
    payload: web::Json<AuthConfig>,
) -> Result<AccessTokenResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/access_token", platform.platform_domain());

    let response = client
        .post(&api_url) // 使用 &api_url 避免所有权问题
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        match response.json::<AccessTokenResponse>().await {
            Ok(body) => {
                debug!("响应体: {:?}", &body);
                Ok(body)
            }
            Err(e) => Err(Box::new(e)),
        }
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取响应体".to_string());

        debug!("请求失败，状态码: {}", status);
        debug!("服务器错误详情: {}", error_body);

        let reason = format!(
            "Token API 调用失败。状态码: {}，服务器详情: {}",
            status, error_body
        );
        Err(reason.into())
    }
}

pub async fn get_access_token_from_cache<T: AsRef<Path>>(
    file_path: T,
) -> Result<AccessToken, Box<dyn Error>> {
    // 1. 安全地检查文件是否存在，并处理 IO 错误
    let file_exists = match tokio::fs::metadata(&file_path).await {
        // 文件不存在 (ErrorKind::NotFound)
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("本地 Token 缓存文件 {:?} 不存在!", file_path.as_ref());
            // 返回一个表示“需要网络获取”的错误，而不是直接返回文件不存在
            return Err("文件不存在，需要获取".into());
        }

        // 其他文件 IO 错误 (权限不足、路径错误等)
        Err(e) => {
            error!("检查文件元数据失败: {}", e);
            // 立即返回 IO 错误
            return Err(e.into());
        }

        // 文件存在 (Ok)
        Ok(_) => {
            debug!(
                "本地 Token 文件{:?} 缓存存在，尝试读取...",
                file_path.as_ref()
            );
            true // 文件存在，继续下一步
        }
    };

    match async_read_and_deserialize::<_, AccessToken>(&file_path).await {
        Ok(config) => {
            debug!("异步解析文件成功! ==>{:?}", &config);

            // 3. 检查是否过期
            // 注意：过期时间 - 当前时间 <= 0 则表示已过期
            if config.expired_at <= Utc::now() {
                debug!("当前token已经过期，尝试重新获取");
                // 返回一个表示“内容过期”的错误
                Err("文件存在，但是内容过期了".into())
            } else {
                // 未过期，返回 Token
                Ok(config)
            }
        }

        Err(e) => {
            error!("异步解析文件失败: {}。退回到网络请求。", e);
            // 返回解析错误，让调用者知道需要网络请求
            Err(e.into())
        }
    }
}
/// 获取访问需要的access_token
pub async fn access_token_and_cache(
    payload: web::Json<AuthConfig>,
    env: web::Data<NetDiskEnv>,
) -> Result<AccessTokenResponse, Box<dyn Error>> {
    let file_path = env.config_dir.clone().join("config.toml");
    let mut body: AccessTokenResponse;
    match get_access_token_from_cache(&file_path).await {
        Ok(access) => {
            //TODO 此处构造逻辑中xtrace有点问题
            body = AccessTokenResponse::new(
                200,
                "响应成功".to_string(),
                access.clone(),
                "xtrace".to_string(),
            );
        }
        Err(_) => {
            debug!(
                "从配置文件{:?}获取配置失败,尝试通过接口获取....",
                &file_path
            );
            body = access_token(payload)
                .await
                .map_err(|e| Box::<dyn Error>::from(e.to_string()))?;

            let token_for_save = body.data.clone();
            let _ = async_write_toml(token_for_save, file_path).await;
            debug!("新的配置文件更新完毕!");
        }
    }
    Ok(body)
}

#[get("/files_query")]
pub async fn files_query(
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
    // debug!("mesg = {:?}", &response.text().await);
    // Err("功能未完成".into())

    // 解析响应
    let api_response: FileResponse = response
        .json()
        .await
        .map_err(|e| format!("响应解析失败: {}", e))?;

    Ok(HttpResponse::Ok().json(api_response))
}
