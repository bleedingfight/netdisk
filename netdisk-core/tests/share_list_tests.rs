//! share/list API 集成测试
//!
//! 测试获取分享列表接口
//! 对应 curl: curl -X GET http://127.0.0.1:8080/share/list?limit=10

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::share_file_api::share_list_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fixtures::ShareFixtures;
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
                .route("/share/list", web::get().to(share_list_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试获取分享列表成功
#[actix_web::test]
async fn test_share_list_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_list(ShareFixtures::share_list_response(5))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/share/list?limit=10")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["shareList"].is_array());
    assert_eq!(body["data"]["shareList"].as_array().unwrap().len(), 5);
}

/// 测试空分享列表
#[actix_web::test]
async fn test_share_list_empty() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_list(ShareFixtures::share_list_response(0))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/share/list?limit=10")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert!(body["data"]["shareList"].as_array().unwrap().is_empty());
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试缺少 limit 参数
#[actix_web::test]
async fn test_share_list_missing_limit() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/share/list")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing limit should return 400");
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_share_list_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/share/list")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/share/list?limit=10")
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
async fn test_share_list_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_list(ShareFixtures::share_list_response(2))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/share/list?limit=10")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    let data = &body["data"];
    assert!(data.get("shareList").is_some(), "Data should have 'shareList'");
}
