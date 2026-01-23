use crate::io_basic::read_and_write::*;
use crate::netdisk_api::api_client::ApiClient;
use crate::netdisk_auth::basic_env::NetDiskEnv;
use crate::responses::prelude::*;
use crate::crypto::TokenCrypto;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use log::{debug, error, info};
use reqwest;
use std::error::Error;
use std::path::Path;

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
    let _file_exists = match tokio::fs::metadata(&file_path).await {
        // 文件不存在 (ErrorKind::NotFound)
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("本地 Token 缓存文件 {:?} 不存在!", file_path.as_ref());
            // 返回一个表示"需要网络获取"的错误，而不是直接返回文件不存在
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

    // 2. 尝试读取加密格式 (SecureToken)
    if let Ok(secure_token) = async_read_and_deserialize::<_, SecureToken>(&file_path).await {
        debug!("读取到加密 Token 格式");
        
        // 检查是否过期
        if secure_token.is_expired() {
            debug!("当前 token 已经过期，尝试重新获取");
            return Err("Token 已过期".into());
        }
        
        // 解密并返回
        let access_token = secure_token.to_access_token();
        debug!("Token 解密成功");
        return Ok(access_token);
    }

    // 3. 回退：尝试读取旧的明文格式 (AccessToken)
    match async_read_and_deserialize::<_, AccessToken>(&file_path).await {
        Ok(config) => {
            debug!("读取到明文 Token 格式 (旧格式)");

            // 检查是否过期
            if config.expired_at <= Utc::now() {
                debug!("当前 token 已经过期，尝试重新获取");
                Err("文件存在，但是内容过期了".into())
            } else {
                // 未过期，返回 Token
                // 注意：下次保存时会自动转换为加密格式
                Ok(config)
            }
        }

        Err(e) => {
            error!("解析 Token 文件失败: {}。退回到网络请求。", e);
            Err(e.into())
        }
    }
}

/// 获取访问需要的access_token
/// 
/// 如果设置了 NETDISK_ENCRYPTION_KEY 环境变量，
/// token 会以加密形式存储在本地缓存文件中。
pub async fn access_token_and_cache(
    payload: web::Json<AuthConfig>,
    env: web::Data<NetDiskEnv>,
) -> Result<AccessTokenResponse, Box<dyn Error>> {
    let file_path = env.config_dir.clone().join("config.toml");
    let body: AccessTokenResponse;
    
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

            // 使用加密格式保存 token
            if let Some(token_for_save) = body.data.clone() {
                let file_path_owned = file_path.clone();
                if TokenCrypto::is_available() {
                    // 加密存储
                    let secure_token = SecureToken::from_access_token(&token_for_save);
                    debug!("使用加密格式保存 token");
                    let _ = async_write_toml(secure_token, file_path_owned).await;
                } else {
                    // 明文存储 (警告)
                    debug!("警告: 未设置加密密钥，token 将以明文存储");
                    let _ = async_write_toml(token_for_save, file_path_owned).await;
                }
            }
            debug!("新的配置文件更新完毕!");
        }
    }
    Ok(body)
}

// ============================================================================
// 可测试版本的 API 函数
// ============================================================================

/// 获取 access_token - 可测试版本
/// 
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"client_id":"xxx", "client_secret":"xxx"}' http://127.0.0.1:8080/access_token
pub async fn access_token_v2(
    payload: web::Json<AuthConfig>,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("access_token_v2: 请求获取 token");

    let response = client.http_client()
        .post(client.url("/api/v1/access_token"))
        .header("Platform", client.platform())
        .json(&payload.into_inner())
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if response.status().is_success() {
        let body: AccessTokenResponse = response
            .json()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(body))
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取响应体".to_string());
        Err(actix_web::error::ErrorInternalServerError(format!(
            "Token API 调用失败。状态码: {}，详情: {}",
            status, error_body
        )))
    }
}
