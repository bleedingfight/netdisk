#[cfg(test)]
mod tests {
    use netdisk_core::responses::prelude::*;
    use netdisk_core::create_app;
    use std::env;
    use std::fs;
    use actix_web::http;
    use tempfile::TempDir;
    use netdisk_core::netdisk_auth::basic_env::NetDiskEnv;
    use netdisk_core::netdisk_api::prelude::*;
    use actix_web::{test,web,App};
use actix_web::{web, App};
use serde_json::json;
use your_crate_name::{download, AccessToken, FileQuery, DownloadUrlResponse}; // ⚠️ 替换

#[actix_web::test]
async fn test_download_handler() {
    let token = web::Data::new(AccessToken {
        access_token: "test_token".to_string(),
    });

    // 构造查询参数
    let query = web::Query(FileQuery {
        fileId: "18340536".to_string(),
    });

    // 直接调用 handler，不走 Actix 服务
    let result = download(query, token).await;

    match result {
        Ok(resp) => {
            // 提取 HTTP 响应体
            let body_bytes = actix_web::body::to_bytes(resp.into_body())
                .await
                .expect("读取响应体失败");
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
            println!("响应内容: {}", body_str);

            // 尝试解析为 DownloadUrlResponse
            if let Ok(parsed) = serde_json::from_str::<DownloadUrlResponse>(&body_str) {
                println!("解析成功: {:?}", parsed);
            } else {
                println!("响应体不是 DownloadUrlResponse 格式");
            }
        }
        Err(e) => {
            panic!("调用 download 失败: {}", e);
        }
    }
}

    // #[actix_web::test]
    // async fn test_file_download() {
    //     let env = NetDiskEnv::new().expect("初始化 NetDiskEnv 失败");
    //     let access_token = AccessToken::default();

    //     let app = test::init_service(
    //         create_app(web::Data::new(env), web::Data::new(access_token))
    //     ).await;

    //     let app = test::init_service(
    //         App::new()
    //             .service(web::resource("/download").route(web::get().to(download)))
    //     ).await;

    //     // 2. 构建请求，模拟 curl 命令的行为
    //     let req = test::TestRequest::get() // 模拟 -X GET
    //         // 模拟 URI 和查询参数 (?fileId=18340536)
    //         .uri("/download?fileId=18340536") 
    //         // 模拟 -H 'Content-Type: application/json'
    //         // 注意：GET 请求通常忽略 Body，但 Actix 会检查 Header
    //         .insert_header((http::header::CONTENT_TYPE, "application/json"))
    //         .to_request();
    //     println!("===============================");

    //     // 3. 发送请求并获取响应
    //     let resp = test::call_service(&app, req).await;

    //     // 4. 断言检查

    //     // 检查状态码是否为 200 OK
    //     assert!(resp.status().is_success()); 
    //     
    //     // 检查响应体内容 (可选)
    //     let response_body = test::read_body(resp).await;
    //     assert_eq!(response_body, web::Bytes::from("Mock file content for ID: 18340536"));
    // }
}
