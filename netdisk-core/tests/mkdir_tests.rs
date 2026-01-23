//! mkdir API 集成测试
//!
//! 测试创建目录接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' \
//!   -d '{"name":"新文件夹", "parentID":0}' http://127.0.0.1:8080/file/mkdir

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_api::mkdir_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
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
                .route("/file/mkdir", web::post().to(mkdir_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试在根目录创建文件夹
/// 对应 curl: curl -X POST -H 'Content-Type: application/json' \
///   -d '{"name":"新文件夹", "parentID":0}' http://127.0.0.1:8080/file/mkdir
#[actix_web::test]
async fn test_mkdir_in_root_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::mkdir(FileFixtures::mkdir_response(123456))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "新文件夹",
            "parentID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["dirID"].is_number());
}

/// 测试在子目录创建文件夹
#[actix_web::test]
async fn test_mkdir_in_subfolder_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::mkdir(FileFixtures::mkdir_response(789012))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 在子目录 (parentID = 12345) 创建
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "子目录文件夹",
            "parentID": 12345
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
}

/// 测试创建中文名称文件夹
#[actix_web::test]
async fn test_mkdir_chinese_name() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::mkdir(FileFixtures::mkdir_response(111222))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "中文文件夹名称测试",
            "parentID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

/// 测试创建带特殊字符的文件夹名
#[actix_web::test]
async fn test_mkdir_special_characters() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::mkdir(FileFixtures::mkdir_response(333444))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 特殊字符但合法的名称
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "文件夹-2024_备份",
            "parentID": 0
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
async fn test_mkdir_empty_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Empty body should return 400");
}

/// 测试缺少 name 字段
#[actix_web::test]
async fn test_mkdir_missing_name() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 name 字段
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "parentID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing name should return 400");
}

/// 测试缺少 parentID 字段
#[actix_web::test]
async fn test_mkdir_missing_parent_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少 parentID 字段
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "新文件夹"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Missing parentID should return 400");
}

/// 测试无效 JSON
#[actix_web::test]
async fn test_mkdir_invalid_json() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid JSON should return 400");
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_mkdir_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/upload/v1/file/mkdir")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "测试文件夹",
            "parentID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

/// 测试目录名称重复错误
#[actix_web::test]
async fn test_mkdir_duplicate_name() {
    // Arrange
    let ctx = TestContext::new().await;

    // 模拟目录名称已存在的错误响应
    Mock::given(method("POST"))
        .and(path("/upload/v1/file/mkdir"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "code": 400,
                    "message": "目录名称已存在",
                    "data": null,
                    "x-traceID": "trace-test-duplicate"
                }))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "已存在的文件夹",
            "parentID": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - API 返回 200 但业务码非 0
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 400);
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应结构包含所有必需字段
#[actix_web::test]
async fn test_mkdir_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::mkdir(FileFixtures::mkdir_response(999999))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/mkdir")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({
            "name": "结构测试",
            "parentID": 0
        }))
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
    assert!(data.get("dirID").is_some(), "Data should have 'dirID'");
}
