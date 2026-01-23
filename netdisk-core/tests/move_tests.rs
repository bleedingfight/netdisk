//! move API 集成测试
//!
//! 测试文件移动接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' \
//!   -d '{"fileIDs":[123,456], "toParentFileID":789}' http://127.0.0.1:8080/file/move

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_move_api::move_file_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::{method, path};
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
                .route("/file/move", web::post().to(move_file_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试移动单个文件
#[actix_web::test]
async fn test_move_single_file_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_move(FileFixtures::file_move_response())
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [12345],
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
}

/// 测试移动多个文件
#[actix_web::test]
async fn test_move_multiple_files_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_move(FileFixtures::file_move_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 移动多个文件
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [100, 200, 300, 400, 500],
            "toParentFileID": 999
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
}

/// 测试移动文件到根目录
#[actix_web::test]
async fn test_move_to_root() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_move(FileFixtures::file_move_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 移动到根目录 (toParentFileID = 0)
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [12345],
            "toParentFileID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试空请求体
#[actix_web::test]
async fn test_move_empty_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Empty body should return 400");
}

/// 测试缺少 fileIDs 字段
#[actix_web::test]
async fn test_move_missing_file_ids() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing fileIDs should return 400");
}

/// 测试缺少 toParentFileID 字段
#[actix_web::test]
async fn test_move_missing_to_parent_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [12345]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing toParentFileID should return 400");
}

/// 测试空 fileIDs 数组
#[actix_web::test]
async fn test_move_empty_file_ids() {
    // Arrange
    let ctx = TestContext::new().await;

    // 模拟 API 对空数组的响应 (可能是业务错误)
    Mock::given(path("/api/v1/file/move"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "code": 400,
                    "message": "文件ID列表不能为空",
                    "data": null,
                    "x-traceID": "trace-empty-ids"
                }))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [],
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - API 返回 200 但业务码非 0
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 400);
}

/// 测试无效 JSON
#[actix_web::test]
async fn test_move_invalid_json() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid JSON should return 400");
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_move_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/file/move")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [12345],
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

/// 测试文件不存在错误
#[actix_web::test]
async fn test_move_file_not_found() {
    // Arrange
    let ctx = TestContext::new().await;

    Mock::given(path("/api/v1/file/move"))
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
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [99999999],
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 404);
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应结构包含所有必需字段
#[actix_web::test]
async fn test_move_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_move(FileFixtures::file_move_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/move")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIDs": [12345],
            "toParentFileID": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    // 验证顶层字段
    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");
    // move API 的 data 可能是 null
}
