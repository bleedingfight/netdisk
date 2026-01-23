//! file_query API 集成测试
//!
//! 使用 wiremock 模拟 123云盘 API，测试 file_query 接口的各种场景

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_api::file_query_v2;
use netdisk_core::responses::prelude::*;
use wiremock::MockServer;

use crate::fixtures::FileFixtures;
use crate::mock_server::NetdiskMock;

/// 测试上下文，封装测试所需的所有资源
struct TestContext {
    mock_server: MockServer,
    token: AccessToken,
    client: ApiClient,
}

impl TestContext {
    /// 创建新的测试上下文
    async fn new() -> Self {
        let mock_server = MockServer::start().await;
        let token = AccessToken::new(
            "test_access_token".to_string(),
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let client = ApiClient::with_config(ApiClientConfig::for_test(mock_server.uri()));

        Self {
            mock_server,
            token,
            client,
        }
    }

    /// 创建配置好的测试服务
    async fn service(
        &self,
    ) -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        test::init_service(
            App::new()
                .app_data(web::Data::new(self.token.clone()))
                .app_data(web::Data::new(self.client.clone()))
                .route("/file/file_query", web::get().to(file_query_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

#[actix_web::test]
async fn test_file_query_success() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;
    
    // 设置 mock 响应
    NetdiskMock::file_query(file_id, FileFixtures::file_query_success_response(file_id as u64))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/file_query?fileID={}", file_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"].is_object());
    assert_eq!(body["data"]["fileID"], file_id);
    assert_eq!(body["data"]["filename"], "test_file.txt");
}

#[actix_web::test]
async fn test_file_query_with_different_file_ids() {
    // 测试不同的文件 ID
    let test_cases: Vec<(i64, &str)> = vec![
        (1_i64, "最小有效 ID"),
        (999999_i64, "普通 ID"),
        (i64::MAX / 2, "大 ID"),  // 使用较小的值避免溢出
    ];

    for (file_id, description) in test_cases {
        let ctx = TestContext::new().await;
        
        NetdiskMock::file_query_any(FileFixtures::file_query_success_response(file_id as u64))
            .mount(&ctx.mock_server)
            .await;

        let app = ctx.service().await;
        let req = test::TestRequest::get()
            .uri(&format!("/file/file_query?fileID={}", file_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(
            resp.status().is_success(),
            "{}: Expected success, got: {}",
            description,
            resp.status()
        );
    }
}

// ============================================================================
// 错误场景测试
// ============================================================================

#[actix_web::test]
async fn test_file_query_file_not_found() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 99999_i64;

    NetdiskMock::file_query(file_id, FileFixtures::file_not_found_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/file_query?fileID={}", file_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - API 返回 200 但 code 不为 0
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 404);
    assert_eq!(body["message"], "file not found");
}

#[actix_web::test]
async fn test_file_query_auth_failed() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;

    NetdiskMock::file_query(file_id, FileFixtures::auth_failed_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/file_query?fileID={}", file_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 401);
}

#[actix_web::test]
async fn test_file_query_missing_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 不传递 fileID 参数
    let req = test::TestRequest::get()
        .uri("/file/file_query")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 Bad Request
    assert_eq!(
        resp.status().as_u16(),
        400,
        "Missing parameter should return 400"
    );
}

#[actix_web::test]
async fn test_file_query_invalid_file_id_format() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 传递非数字的 fileID
    let req = test::TestRequest::get()
        .uri("/file/file_query?fileID=not_a_number")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 Bad Request
    assert_eq!(
        resp.status().as_u16(),
        400,
        "Invalid parameter format should return 400"
    );
}

// ============================================================================
// 服务器错误场景测试
// ============================================================================

#[actix_web::test]
async fn test_file_query_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/file/detail")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/file_query?fileID=12345")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 500 Internal Server Error
    assert_eq!(
        resp.status().as_u16(),
        500,
        "Server error should be propagated"
    );
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

#[actix_web::test]
async fn test_file_query_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;

    NetdiskMock::file_query(file_id, FileFixtures::file_query_success_response(file_id as u64))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/file_query?fileID={}", file_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert - 验证响应结构
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    // 验证顶层字段
    assert!(body.get("code").is_some(), "Response should have 'code' field");
    assert!(body.get("message").is_some(), "Response should have 'message' field");
    assert!(body.get("data").is_some(), "Response should have 'data' field");
    assert!(body.get("x-traceID").is_some(), "Response should have 'x-traceID' field");
    
    // 验证 data 字段结构
    let data = &body["data"];
    assert!(data.get("fileID").is_some(), "Data should have 'fileID' field");
    assert!(data.get("filename").is_some(), "Data should have 'filename' field");
    assert!(data.get("size").is_some(), "Data should have 'size' field");
    assert!(data.get("etag").is_some(), "Data should have 'etag' field");
}

// ============================================================================
// Mock 验证测试
// ============================================================================

#[actix_web::test]
async fn test_file_query_calls_correct_endpoint() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;

    NetdiskMock::file_query(file_id, FileFixtures::file_query_success_response(file_id as u64))
        .expect(1)  // 期望被调用 1 次
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/file_query?fileID={}", file_id))
        .to_request();
    let _ = test::call_service(&app, req).await;

    // Assert - wiremock 会自动验证 expect(1)
    // 如果 mock 没有被调用恰好 1 次，测试会失败
}
