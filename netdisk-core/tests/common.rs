//! 测试公共模块
//!
//! 提供测试所需的共享基础设施

use actix_web::test;

/// 测试请求构建器
/// 简化测试请求的构建过程
pub struct TestRequestBuilder;

impl TestRequestBuilder {
    /// 构建 GET 请求
    pub fn get(uri: &str) -> test::TestRequest {
        test::TestRequest::get()
            .uri(uri)
            .insert_header(("Content-Type", "application/json"))
    }

    /// 构建 POST 请求
    pub fn post(uri: &str) -> test::TestRequest {
        test::TestRequest::post()
            .uri(uri)
            .insert_header(("Content-Type", "application/json"))
    }

    /// 构建带 JSON body 的 POST 请求
    pub fn post_json<T: serde::Serialize>(uri: &str, body: &T) -> test::TestRequest {
        test::TestRequest::post()
            .uri(uri)
            .insert_header(("Content-Type", "application/json"))
            .set_json(body)
    }
}

/// 响应断言辅助
pub struct ResponseAssert;

impl ResponseAssert {
    /// 断言响应成功并返回 JSON 体
    pub async fn assert_ok_json(
        resp: actix_web::dev::ServiceResponse,
    ) -> serde_json::Value {
        assert!(
            resp.status().is_success(),
            "Expected success status, got: {}",
            resp.status()
        );
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).expect("Response should be valid JSON")
    }

    /// 断言响应包含指定的 code
    pub async fn assert_api_code(
        resp: actix_web::dev::ServiceResponse,
        expected_code: i32,
    ) {
        let body = test::read_body(resp).await;
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("Response should be valid JSON");
        
        let actual_code = json
            .get("code")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        
        assert_eq!(
            actual_code,
            Some(expected_code),
            "Expected code {}, got {:?}. Response: {}",
            expected_code,
            actual_code,
            String::from_utf8_lossy(&body)
        );
    }
}
