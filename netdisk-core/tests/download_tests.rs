//! download API 集成测试
//!
//! 测试获取文件下载链接接口
//! 对应 curl: curl -X GET http://127.0.0.1:8080/file/download?fileId=xxx

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_api::download_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::{path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fixtures::FileFixtures;
use crate::mock_server::NetdiskMock;

/// 测试上下文
struct TestContext {
    mock_server: MockServer,
    token: AccessToken,
    client: ApiClient,
}

impl TestContext {
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
                .route("/file/download", web::get().to(download_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试获取下载链接成功
#[actix_web::test]
async fn test_download_success() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;

    NetdiskMock::download(file_id, FileFixtures::download_url_response())
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/download?fileID={}", file_id))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["downloadUrl"].is_string());
}

/// 测试不同的文件 ID
#[actix_web::test]
async fn test_download_different_file_ids() {
    let test_cases = vec![
        (1_i64, "最小 ID"),
        (999999_i64, "普通 ID"),
        (9999999999_i64, "大 ID"),
    ];

    for (file_id, description) in test_cases {
        // Arrange
        let ctx = TestContext::new().await;

        NetdiskMock::download(file_id, FileFixtures::download_url_response())
            .mount(&ctx.mock_server)
            .await;

        let app = ctx.service().await;

        // Act
        let req = test::TestRequest::get()
            .uri(&format!("/file/download?fileID={}", file_id))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Assert
        assert!(
            resp.status().is_success(),
            "Test case '{}' failed: expected success, got {}",
            description,
            resp.status()
        );
    }
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试缺少 fileID 参数
#[actix_web::test]
async fn test_download_missing_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 fileID
    let req = test::TestRequest::get()
        .uri("/file/download")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing fileID should return 400");
}

/// 测试无效的 fileID 参数
#[actix_web::test]
async fn test_download_invalid_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 无效的 fileID
    let req = test::TestRequest::get()
        .uri("/file/download?fileID=abc")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid fileID should return 400");
}

/// 测试文件不存在
#[actix_web::test]
async fn test_download_file_not_found() {
    // Arrange
    let ctx = TestContext::new().await;

    Mock::given(path("/api/v1/file/download_info"))
        .and(query_param("fileID", "99999999"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "code": 404,
                    "message": "文件不存在",
                    "data": null,
                    "x-traceID": "trace-not-found"
                }))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/download?fileID=99999999")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 404);
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_download_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/file/download_info")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/download?fileID=12345")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应结构包含所有必需字段
#[actix_web::test]
async fn test_download_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_id = 12345_i64;

    NetdiskMock::download(file_id, FileFixtures::download_url_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri(&format!("/file/download?fileID={}", file_id))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    let data = &body["data"];
    assert!(data.get("downloadUrl").is_some(), "Data should have 'downloadUrl'");
}
