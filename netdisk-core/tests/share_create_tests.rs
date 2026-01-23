//! share/create API 集成测试
//!
//! 测试创建分享链接接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' \
//!   -d '{"shareName":"测试分享", "shareExpire":"7", "fileIDList":"123,456"}' \
//!   http://127.0.0.1:8080/share/create

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::share_file_api::share_create_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::MockServer;

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
                .route("/share/create", web::post().to(share_create_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试创建7天有效期分享链接
#[actix_web::test]
async fn test_share_create_7_days_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_create(ShareFixtures::share_create_response(12345))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "测试分享",
            "shareExpire": "7",
            "fileIDList": "123,456"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["shareID"].is_number());
    assert!(body["data"]["shareKey"].is_string());
}

/// 测试创建永久分享链接
#[actix_web::test]
async fn test_share_create_permanent_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_create(ShareFixtures::share_create_response(99999))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "永久分享",
            "shareExpire": "0",
            "fileIDList": "789"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
}

/// 测试创建带密码的分享链接
#[actix_web::test]
async fn test_share_create_with_password_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_create(ShareFixtures::share_create_response(54321))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "加密分享",
            "shareExpire": "30",
            "fileIDList": "111,222,333",
            "sharePwd": "abcd"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["shareID"], 54321);
}

/// 测试创建带流量控制的分享链接
#[actix_web::test]
async fn test_share_create_with_traffic_control_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_create(ShareFixtures::share_create_response(11111))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "流量控制分享",
            "shareExpire": "1",
            "fileIDList": "999",
            "trafficSwitch": 4,
            "trafficLimitSwitch": 2,
            "trafficLimit": 10737418240_u64
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试缺少必填字段
#[actix_web::test]
async fn test_share_create_missing_required_field() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 shareName
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareExpire": "7",
            "fileIDList": "123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 错误
    assert_eq!(resp.status().as_u16(), 400, "Missing required field should return 400");
}

/// 测试空请求体
#[actix_web::test]
async fn test_share_create_empty_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Empty body should return 400");
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_share_create_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/share/create")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "测试分享",
            "shareExpire": "7",
            "fileIDList": "123"
        }))
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
async fn test_share_create_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::share_create(ShareFixtures::share_create_response(77777))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/share/create")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "shareName": "结构验证分享",
            "shareExpire": "7",
            "fileIDList": "123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    let data = &body["data"];
    assert!(data.get("shareID").is_some(), "Data should have 'shareID'");
    assert!(data.get("shareKey").is_some(), "Data should have 'shareKey'");
}
