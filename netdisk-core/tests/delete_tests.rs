//! delete API 集成测试
//!
//! 测试永久删除文件接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' \
//!   -d '{"fileIds":[123,456]}' http://127.0.0.1:8080/delete

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_delete_api::delete_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::path;
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
                .route("/delete", web::post().to(delete_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试永久删除单个文件
#[actix_web::test]
async fn test_delete_single_file_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_delete(FileFixtures::delete_response())
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": [12345]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
}

/// 测试永久删除多个文件
#[actix_web::test]
async fn test_delete_multiple_files_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_delete(FileFixtures::delete_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": [100, 200, 300, 400, 500]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试空请求体
#[actix_web::test]
async fn test_delete_empty_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Empty body should return 400");
}

/// 测试缺少 fileIds 字段
#[actix_web::test]
async fn test_delete_missing_file_ids() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing fileIds should return 400");
}

/// 测试空 fileIds 数组
#[actix_web::test]
async fn test_delete_empty_file_ids() {
    // Arrange
    let ctx = TestContext::new().await;

    Mock::given(path("/api/v1/file/delete"))
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
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": []
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 400);
}

/// 测试无效 JSON
#[actix_web::test]
async fn test_delete_invalid_json() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid JSON should return 400");
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_delete_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/file/delete")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": [12345]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

/// 测试文件不存在错误
#[actix_web::test]
async fn test_delete_file_not_found() {
    // Arrange
    let ctx = TestContext::new().await;

    Mock::given(path("/api/v1/file/delete"))
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
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": [99999999]
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
async fn test_delete_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_delete(FileFixtures::delete_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/delete")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "fileIds": [12345]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");
}
