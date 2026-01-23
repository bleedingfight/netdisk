//! user_info API 集成测试
//!
//! 测试获取用户信息接口
//! 对应 curl: curl -X GET http://127.0.0.1:8080/user_info

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::user_info_api::user_info_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fixtures::UserFixtures;
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
                .route("/user_info", web::get().to(user_info_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试获取用户信息成功
#[actix_web::test]
async fn test_user_info_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::user_info(UserFixtures::user_info_response())
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/user_info")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["uid"].is_number());
    assert!(body["data"]["nickname"].is_string());
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试认证失败
#[actix_web::test]
async fn test_user_info_auth_failed() {
    // Arrange
    let ctx = TestContext::new().await;

    Mock::given(path("/api/v1/user/info"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "code": 401,
                    "message": "token 无效或已过期",
                    "data": null,
                    "x-traceID": "trace-auth-failed"
                }))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/user_info")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 401);
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_user_info_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/user/info")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/user_info")
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
async fn test_user_info_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::user_info(UserFixtures::user_info_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/user_info")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    // 验证顶层字段
    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    // 验证 data 字段结构
    let data = &body["data"];
    assert!(data.get("uid").is_some(), "Data should have 'uid'");
    assert!(data.get("nickname").is_some(), "Data should have 'nickname'");
    assert!(data.get("spaceUsed").is_some(), "Data should have 'spaceUsed'");
    assert!(data.get("spacePermanent").is_some(), "Data should have 'spacePermanent'");
}
