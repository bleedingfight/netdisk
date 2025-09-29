use crate::netdisk_auth::config::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use reqwest;
use serde_json::json;
use std::error::Error;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[post("/access_token")]
pub async fn access_token(info: web::Json<Config>) -> Result<AccessTokenResponse, Box<dyn Error>> {
    // 移除第二次实例化
    let client = reqwest::Client::new();

    let payload_config = AuthConfig::new(
        info.client_id().to_string(),
        info.client_secret().to_string(),
    );
    let platform = PlatformConfig::default();
    let api_url = format!("https://{}/api/v1/access_token", platform.platform_domain());

    let payload = json!({
        "clientId": payload_config.client_id(),
        "clientSecret": payload_config.client_secret()
    });

    let response = client
        .post(&api_url) // 使用 &api_url 避免所有权问题
        .header("Platform", platform.platform())
        .json(&payload)
        .send()
        .await?; // 自动处理 reqwest::Error

    if response.status().is_success() {
        // 成功，直接解析并返回真实数据
        println!("请求成功");
        // let body = response.json::<AccessTokenResponse>().await?;
        match response.json::<AccessTokenResponse>().await {
            Ok(body) => {
                println!("响应体: {:?}", body);
                let token_response = AccessTokenResponse::new(body.access_token, body.expires_at);
                return Ok(token_response);
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    } else {
        // 失败，构造详细错误信息
        let status = response.status();

        // 尝试获取服务器返回的错误文本，若失败则提供默认信息
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取响应体".to_string());

        println!("请求失败，状态码: {}", status);
        println!("服务器错误详情: {}", error_body);

        let reason = format!(
            "Token API 调用失败。状态码: {}，服务器详情: {}",
            status, error_body
        );
        return Err(reason.into());
    }
}
