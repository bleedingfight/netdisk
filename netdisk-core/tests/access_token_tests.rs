//! access_token API 集成测试
//!
//! 测试获取访问令牌接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' \
//!   -d '{"client_id":"xxx", "client_secret":"xxx"}' http://127.0.0.1:8080/access_token

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::auth_api::access_token_v2;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fixtures::TokenFixtures;
use crate::mock_server::NetdiskMock;

/// 测试上下文
struct TestContext {
    mock_server: MockServer,
    client: ApiClient,
}

impl TestContext {
    async fn new() -> Self {
        let mock_server = MockServer::start().await;
        let client = ApiClient::with_config(ApiClientConfig::for_test(mock_server.uri()));

        Self { mock_server, client }
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
                .app_data(web::Data::new(self.client.clone()))
                .route("/access_token", web::post().to(access_token_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试获取 access_token 成功
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"client_id":"123", "client_secret":"123"}' http://127.0.0.1:8080/access_token
#[actix_web::test]
async fn test_access_token_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::access_token(TokenFixtures::valid_token_response())
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "test_client_id",
            "client_secret": "test_client_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["accessToken"].is_string());
    assert!(body["data"]["expiredAt"].is_string());
}

/// 测试使用环境变量中的凭证
#[actix_web::test]
async fn test_access_token_with_env_credentials() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::access_token(TokenFixtures::valid_token_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 模拟使用环境变量的 curl 命令
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "env_client_id",
            "client_secret": "env_client_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试无效的凭证
#[actix_web::test]
async fn test_access_token_invalid_credentials() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::access_token(TokenFixtures::invalid_credentials_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "wrong_id",
            "client_secret": "wrong_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - API 返回 200 但业务码非 0
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 401);
    assert!(body["message"].as_str().unwrap().contains("invalid"));
}

/// 测试缺少 client_id
#[actix_web::test]
async fn test_access_token_missing_client_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 client_id
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_secret": "test_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing client_id should return 400");
}

/// 测试缺少 client_secret
#[actix_web::test]
async fn test_access_token_missing_client_secret() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 client_secret
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "test_id"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing client_secret should return 400");
}

/// 测试空请求体
#[actix_web::test]
async fn test_access_token_empty_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Empty body should return 400");
}

/// 测试无效 JSON
#[actix_web::test]
async fn test_access_token_invalid_json() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid JSON should return 400");
}

// ============================================================================
// 服务器错误测试
// ============================================================================

/// 测试服务器内部错误
#[actix_web::test]
async fn test_access_token_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/access_token")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "test_id",
            "client_secret": "test_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

// ============================================================================
// 请求验证测试
// ============================================================================

/// 验证请求格式正确发送到上游 API
#[actix_web::test]
async fn test_access_token_request_format() {
    // Arrange
    let ctx = TestContext::new().await;

    // 精确匹配请求
    Mock::given(method("POST"))
        .and(path("/api/v1/access_token"))
        .and(header("Platform", "open_platform"))
        .and(body_json(json!({
            "client_id": "my_client_id",
            "client_secret": "my_client_secret"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(TokenFixtures::valid_token_response()),
        )
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "my_client_id",
            "client_secret": "my_client_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应包含所有必需字段
#[actix_web::test]
async fn test_access_token_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::access_token(TokenFixtures::valid_token_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/access_token")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "client_id": "test_id",
            "client_secret": "test_secret"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    // 顶层字段
    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    // data 字段结构
    let data = &body["data"];
    assert!(data.get("accessToken").is_some(), "Data should have 'accessToken'");
    assert!(data.get("expiredAt").is_some(), "Data should have 'expiredAt'");
}
