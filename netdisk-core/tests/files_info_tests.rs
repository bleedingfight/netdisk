//! files_info API 集成测试
//!
//! 测试批量文件信息查询接口
//! 对应 curl: curl -X POST -H 'Content-Type: application/json' -d '{"fileIds":[18226271]}' http://127.0.0.1:8080/file/files_info

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_api::files_info_v2;
use netdisk_core::responses::prelude::*;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
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
                .route("/file/files_info", web::post().to(files_info_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试单个文件查询 - 对应 curl: -d '{"fileIds":[18226271]}'
#[actix_web::test]
async fn test_files_info_single_file() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_ids = vec![18226271_u64];

    // 模拟 123云盘 API 返回
    NetdiskMock::files_info(FileFixtures::files_info_response(&file_ids))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 模拟 curl 请求
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": [18226271_u64]}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    
    // 验证返回了文件列表
    let file_list = &body["data"]["fileList"];
    assert!(file_list.is_array());
    assert_eq!(file_list.as_array().unwrap().len(), 1);

    assert_eq!(file_list[0]["fileId"], 18226271_u64);
}

/// 测试多个文件查询
#[actix_web::test]
async fn test_files_info_multiple_files() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_ids = vec![18226271_u64, 18226272_u64, 18226273_u64];

    NetdiskMock::files_info(FileFixtures::files_info_response(&file_ids))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": file_ids}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    
    let file_list = &body["data"]["fileList"];
    assert_eq!(file_list.as_array().unwrap().len(), 3);
}

/// 测试空文件列表查询
#[actix_web::test]
async fn test_files_info_empty_list() {
    // Arrange
    let ctx = TestContext::new().await;

    // 模拟返回空列表
    NetdiskMock::files_info(json!({
        "code": 0,
        "message": "success",
        "data": {
            "fileList": []
        },
        "x-traceID": "trace-empty-list"
    }))
    .mount(&ctx.mock_server)
    .await;

    let app = ctx.service().await;

    // Act - 查询空列表
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": []}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    
    let file_list = &body["data"]["fileList"];
    assert!(file_list.as_array().unwrap().is_empty());
}

// ============================================================================
// 错误场景测试
// ============================================================================

/// 测试文件不存在
#[actix_web::test]
async fn test_files_info_file_not_found() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::files_info(json!({
        "code": 404,
        "message": "file not found",
        "data": null,
        "x-traceID": "trace-not-found"
    }))
    .mount(&ctx.mock_server)
    .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": [99999999_u64]}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - API 返回 200 但业务码非 0
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 404);
}

/// 测试认证失败
#[actix_web::test]
async fn test_files_info_unauthorized() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::files_info(FileFixtures::auth_failed_response())
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": [18226271_u64]}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 401);
}

/// 测试缺少请求体
#[actix_web::test]
async fn test_files_info_missing_body() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 不带请求体
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400
    assert_eq!(resp.status().as_u16(), 400, "Missing body should return 400");
}

/// 测试无效的 JSON 格式
#[actix_web::test]
async fn test_files_info_invalid_json() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 无效的 JSON
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 400, "Invalid JSON should return 400");
}

/// 测试错误的字段名
#[actix_web::test]
async fn test_files_info_wrong_field_name() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 使用错误的字段名 file_ids 而不是 fileIds
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"file_ids": [18226271_u64]}))  // 错误的字段名
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400（字段验证失败）
    assert_eq!(resp.status().as_u16(), 400, "Wrong field name should return 400");
}

// ============================================================================
// 服务器错误测试
// ============================================================================

/// 测试服务器内部错误
#[actix_web::test]
async fn test_files_info_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v1/file/infos")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": [18226271_u64]}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500, "Server error should be propagated");
}

// ============================================================================
// 请求验证测试
// ============================================================================

/// 验证请求正确发送到上游 API
#[actix_web::test]
async fn test_files_info_request_format() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_ids = vec![18226271_u64, 18226272_u64];

    // 使用更精确的 mock 匹配，验证请求格式
    Mock::given(method("POST"))
        .and(path("/api/v1/file/infos"))
        .and(header("Authorization", "Bearer test_access_token"))
        .and(header("Platform", "open_platform"))
        .and(body_json(json!({"fileIds": file_ids})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(FileFixtures::files_info_response(&file_ids)),
        )
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": file_ids}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    // wiremock 会自动验证 expect(1) 和请求匹配
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应包含所有必需字段
#[actix_web::test]
async fn test_files_info_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;
    let file_ids = vec![18226271_u64];

    NetdiskMock::files_info(FileFixtures::files_info_response(&file_ids))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::post()
        .uri("/file/files_info")
        .insert_header(("Content-Type", "application/json"))
        .set_json(json!({"fileIds": file_ids}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 验证响应结构
    let body: serde_json::Value = test::read_body_json(resp).await;

    // 顶层字段
    assert!(body.get("code").is_some(), "Should have 'code'");
    assert!(body.get("message").is_some(), "Should have 'message'");
    assert!(body.get("data").is_some(), "Should have 'data'");
    assert!(body.get("x-traceID").is_some(), "Should have 'x-traceID'");

    // data.fileList 结构
    let file_list = &body["data"]["fileList"];
    assert!(file_list.is_array(), "fileList should be array");

    if let Some(first_file) = file_list.as_array().and_then(|arr| arr.first()) {
        assert!(first_file.get("fileId").is_some(), "File should have 'fileId'");
        assert!(first_file.get("filename").is_some(), "File should have 'filename'");
        assert!(first_file.get("size").is_some(), "File should have 'size'");
        assert!(first_file.get("etag").is_some(), "File should have 'etag'");
        assert!(first_file.get("parentFileId").is_some(), "File should have 'parentFileId'");
        assert!(first_file.get("type").is_some(), "File should have 'type'");
        assert!(first_file.get("status").is_some(), "File should have 'status'");
    }
}
