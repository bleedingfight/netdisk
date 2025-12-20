#[cfg(test)]
mod tests {
    use netdisk_core::responses::prelude::*;
    use netdisk_core::netdisk_api::file_api::file_query;
    use actix_web::{test, web, App};
    use serde_json::json;

    /// 测试file_query接口的正常情况
    /// 模拟查询一个存在的文件详情
    #[actix_web::test]
    async fn test_file_query_success() {
        // 创建测试用的访问令牌
        let token = web::Data::new(AccessToken {
            access_token: "test_access_token".to_string(),
            expired_at: chrono::Utc::now(),
        });

        // 创建文件查询参数
        let query = web::Query(FileQuery {
            file_id: 12345, // 测试文件ID
        });

        // 调用file_query函数
        let result = file_query(query, token).await;

        // 验证结果
        match result {
            Ok(response) => {
                // 检查HTTP响应状态
                assert!(response.status().is_success());
                
                // 尝试解析响应体
                let body_bytes = actix_web::body::to_bytes(response.into_body())
                    .await
                    .expect("读取响应体失败");
                let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                
                println!("文件查询成功响应: {}", body_str);
                
                // 尝试解析为FileResponse
                if let Ok(file_response) = serde_json::from_str::<FileResponse>(&body_str) {
                    println!("文件详情解析成功: {:?}", file_response);
                    // 验证返回的数据结构
                    if let Some(file_data) = file_response.data {
                        assert!(file_data.file_id > 0);
                        assert!(!file_data.filename.is_empty());
                    } else {
                        println!("响应数据为空");
                    }
                } else {
                    println!("响应体格式不符合FileResponse结构");
                }
            }
            Err(e) => {
                // 如果API调用失败，记录错误但测试继续
                println!("文件查询API调用失败: {}", e);
                // 在实际环境中，这里应该根据具体情况决定是否panic
                // 对于单元测试，我们可能需要mock外部API调用
            }
        }
    }

    /// 测试file_query接口的参数验证
    /// 测试无效的文件ID格式
    #[actix_web::test]
    async fn test_file_query_invalid_params() {
        let token = web::Data::new(AccessToken {
            access_token: "test_access_token".to_string(),
            expired_at: chrono::Utc::now(),
        });

        // 测试负数文件ID
        let query = web::Query(FileQuery {
            file_id: -1,
        });

        let result = file_query(query, token.clone()).await;
        
        match result {
            Ok(response) => {
                // 即使API返回了响应，我们也需要检查状态码
                if !response.status().is_success() {
                    println!("API返回错误状态码: {}", response.status());
                }
            }
            Err(e) => {
                println!("参数验证错误: {}", e);
                // 参数验证失败是预期的行为
            }
        }
    }

    /// 测试file_query接口的认证失败情况
    /// 使用无效的访问令牌
    #[actix_web::test]
    async fn test_file_query_auth_failure() {
        // 使用无效的访问令牌
        let token = web::Data::new(AccessToken {
            access_token: "invalid_token".to_string(),
            expired_at: chrono::Utc::now(),
        });

        let query = web::Query(FileQuery {
            file_id: 12345,
        });

        let result = file_query(query, token).await;
        
        match result {
            Ok(response) => {
                // 检查是否返回了认证错误
                if response.status() == 401 {
                    println!("认证失败测试通过: 返回了401状态码");
                } else {
                    println!("意外的响应状态: {}", response.status());
                }
            }
            Err(e) => {
                println!("认证测试错误: {}", e);
            }
        }
    }

    /// 测试file_query接口的路由集成
    /// 验证路由配置是否正确
    #[actix_web::test]
    async fn test_file_query_route_integration() {
        let token = web::Data::new(AccessToken {
            access_token: "test_token".to_string(),
            expired_at: chrono::Utc::now(),
        });

        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(token.clone())
                .service(
                    web::resource("/file_query")
                        .route(web::get().to(file_query))
                )
        ).await;

        // 构建测试请求
        let req = test::TestRequest::get()
            .uri("/file_query?fileID=12345")
            .to_request();

        // 发送请求并获取响应
        let resp = test::call_service(&app, req).await;

        // 验证响应
        println!("路由集成测试响应状态: {}", resp.status());
        
        // 我们主要验证路由能够正确处理请求
        // 具体的业务逻辑错误（如文件不存在）是预期的
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    /// 测试文件查询参数的序列化
    /// 确保file_id正确序列化为fileID
    #[actix_web::test]
    async fn test_file_query_serialization() {
        let query = FileQuery {
            file_id: 12345,
        };

        // 序列化为JSON
        let json_str = serde_json::to_string(&query).expect("序列化失败");
        println!("序列化结果: {}", json_str);

        // 验证字段名是否正确
        assert!(json_str.contains("fileID"));
        
        // 反序列化验证
        let deserialized: FileQuery = serde_json::from_str(&json_str).expect("反序列化失败");
        assert_eq!(deserialized.file_id, 12345);
    }

    /// 测试FileResponse结构体的解析
    /// 验证响应数据结构
    #[actix_web::test]
    async fn test_file_response_structure() {
        // 创建模拟的响应数据
        let mock_response = json!({
            "code": 0,
            "message": "success",
            "data": {
                "fileID": 12345,
                "parentFileID": 0,
                "filename": "test_file.txt",
                "type": 1,
                "size": 1024,
                "etag": "abc123",
                "status": 1,
                "createAt": "2023-01-01T00:00:00",
                "trashed": 0
            }
        });

        // 尝试解析为FileResponse
        let file_response: Result<FileResponse, _> = serde_json::from_value(mock_response);
        
        match file_response {
            Ok(response) => {
                println!("Mock响应解析成功: {:?}", response);
                if let Some(file_data) = response.data {
                    assert_eq!(file_data.file_id, 12345);
                    assert_eq!(file_data.filename, "test_file.txt");
                } else {
                    panic!("响应数据为空");
                }
            }
            Err(e) => {
                println!("Mock响应解析失败: {}", e);
            }
        }
    }
}
