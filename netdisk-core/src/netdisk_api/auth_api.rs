use crate::io_basic::read_and_write::*;
use crate::netdisk_auth::basic_env::NetDiskEnv;
use crate::responses::prelude::*;
use actix_web::web;
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
