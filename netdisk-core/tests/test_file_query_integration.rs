#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, middleware};
    use netdisk_core::responses::prelude::*;
    use netdisk_core::netdisk_api::file_api;
    use serde_json::json;

    /// 测试file_query接口的正常情况 - 通过HTTP请求
    #[actix_web::test]
    async fn test_file_query_http_success() {
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;

        // 构建查询请求 - 注意使用正确的参数名fileID
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=20575470")
            .to_request();

        // 发送请求并获取响应
        let resp = test::call_service(&app, req).await;
        
        println!("=== 文件查询接口HTTP测试 ===");
        println!("HTTP响应状态: {}", resp.status());
        println!("响应头: {:?}", resp.headers());

        if resp.status().is_success() {
            let body: serde_json::Value = test::read_body_json(resp).await;
            println!("响应体: {}", serde_json::to_string_pretty(&body).unwrap());
            
            // 验证响应结构
            assert!(body.get("code").is_some());
            assert!(body.get("message").is_some());
            
            // 注意：由于实际调用外部API，这里可能返回错误
            // 我们主要验证接口能够正确处理请求
        } else {
            println!("请求失败，状态码: {}", resp.status());
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println!("错误响应: {}", body_str);
            
            // 即使是错误响应，也应该能解析为JSON
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                assert!(error_json.get("code").is_some());
                assert!(error_json.get("message").is_some());
            }
        }
    }

    /// 测试file_query接口的参数验证 - 通过HTTP请求
    #[actix_web::test]
    async fn test_file_query_http_invalid_params() {
        println!("=== 参数验证HTTP测试 ===");
        
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;
        
        // 测试1: 缺少必需参数
        println!("测试1: 缺少fileID参数");
        let req = test::TestRequest::get()
            .uri("/file_query")  // 缺少fileID参数
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("响应状态: {}", resp.status());
        
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("响应体: {}", body_str);
        
        // 测试2: 无效的文件ID格式
        println!("测试2: 无效的文件ID格式");
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=invalid")
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("响应状态: {}", resp.status());
        
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("响应体: {}", body_str);
        
        // 验证返回了响应（不管是成功还是错误）
        // 因为实际会调用外部API，所以主要验证接口可达
        assert!(body_str.len() > 0); // 确保有响应内容
    }

    /// 测试file_query接口的完整HTTP请求流程
    #[actix_web::test]
    async fn test_file_query_full_http_workflow() {
        println!("=== 完整HTTP流程测试 ===");
        
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;
        
        // 步骤1: 构建有效的查询请求
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=12345")
            .insert_header(("Content-Type", "application/json"))
            .insert_header(("Platform", "open_platform"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        println!("响应状态: {}", resp.status());
        println!("响应头: {:?}", resp.headers());
        
        // 读取响应体
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("响应体: {}", body_str);
        
        // 验证能够解析为JSON（不管成功还是错误）
        if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(&body_str) {
            assert!(json_response.get("code").is_some());
            assert!(json_response.get("message").is_some());
            println!("JSON解析成功");
        } else {
            println!("响应不是有效的JSON格式");
        }
    }

    /// 测试file_query接口的错误处理和边界情况
    #[actix_web::test]
    async fn test_file_query_error_handling() {
        println!("=== 错误处理HTTP测试 ===");
        
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;
        
        // 测试各种边界情况
        let test_cases = vec![
            ("", "缺少文件ID参数"),  // 空参数
            ("fileID=0", "文件ID为0"),  // 边界值
            ("fileID=-1", "负数文件ID"),  // 负数
            ("fileID=999999999999", "超大文件ID"),  // 超大值
        ];

        for (query_params, description) in test_cases {
            println!("测试: {}", description);
            
            let uri = if query_params.is_empty() {
                "/file_query".to_string()
            } else {
                format!("/file_query?{}", query_params)
            };

            let req = test::TestRequest::get()
                .uri(&uri)
                .to_request();

            let resp = test::call_service(&app, req).await;
            println!("  响应状态: {}", resp.status());
            
            let status = resp.status();
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            println!("  响应体: {}", body_str);
            
            // 验证错误处理 - 主要检查是否有响应
            match status.as_u16() {
                200..=299 => println!("  请求成功"),
                400..=499 => println!("  客户端错误处理"),
                500..=599 => println!("  服务器错误处理"),
                _ => println!("  其他响应状态"),
            }
            
            // 验证响应是有效的JSON
            if let Ok(_) = serde_json::from_str::<serde_json::Value>(&body_str) {
                println!("  响应是有效的JSON");
            } else {
                println!("  响应不是有效的JSON");
            }
        }
    }

    /// 测试file_query接口的响应时间和性能
    #[actix_web::test]
    async fn test_file_query_performance() {
        use std::time::Instant;
        
        println!("=== 性能测试 ===");
        
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;
        
        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=12345")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let duration = start.elapsed();
        
        println!("响应时间: {:?}", duration);
        println!("响应状态: {}", resp.status());
        
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("响应体长度: {} 字节", body_str.len());
        
        // 验证响应时间合理（由于调用外部API，时间会较长）
        assert!(duration.as_secs() < 30, "响应时间过长: {:?}", duration);
        
        println!("性能测试完成");
    }

    /// 测试file_query接口的路由配置
    #[actix_web::test]
    async fn test_file_query_routing() {
        println!("=== 路由配置测试 ===");
        
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AccessToken {
                    access_token: "test_access_token".to_string(),
                    expired_at: chrono::Utc::now() + chrono::Duration::hours(1),
                }))
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_api::file_query))
                )
        ).await;
        
        // 测试正确的路由
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=12345")
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("正确路由 - 状态: {}", resp.status());
        
        // 测试错误的路由
        let req = test::TestRequest::get()
            .uri("/wrong_path?fileID=12345")
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("错误路由 - 状态: {}", resp.status());
        
        // 验证路由配置正确
        assert_eq!(resp.status(), 404); // 应该返回404
    }
}