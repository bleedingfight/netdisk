//! file_lists_query API 集成测试
//!
//! 测试文件列表查询接口
//! 对应 curl: curl -X GET http://127.0.0.1:8080/file/file_lists_query?parentFileId=0&limit=100

mod common;
mod fixtures;
mod mock_server;

use actix_web::{test, web, App};
use netdisk_core::netdisk_api::api_client::{ApiClient, ApiClientConfig};
use netdisk_core::netdisk_api::file_api::file_lists_query_v2;
use netdisk_core::responses::prelude::*;
use wiremock::matchers::{method, path, query_param};
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
                .route("/file/file_lists_query", web::get().to(file_lists_query_v2)),
        )
        .await
    }
}

// ============================================================================
// 成功场景测试
// ============================================================================

/// 测试查询根目录文件列表
/// 对应 curl: curl -X GET http://127.0.0.1:8080/file/file_lists_query?parentFileId=0&limit=100
#[actix_web::test]
async fn test_file_lists_query_root_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_list(FileFixtures::file_list_response(5))
        .expect(1)
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=100")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success(), "Expected success, got: {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["message"], "success");
    assert!(body["data"]["fileList"].is_array());
    assert_eq!(body["data"]["fileList"].as_array().unwrap().len(), 5);
}

/// 测试查询子目录文件列表
#[actix_web::test]
async fn test_file_lists_query_subfolder_success() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_list(FileFixtures::file_list_response(3))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 查询子目录 (parentFileId = 12345)
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=12345&limit=50")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["fileList"].as_array().unwrap().len(), 3);
}

/// 测试空目录响应
#[actix_web::test]
async fn test_file_lists_query_empty_folder() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_list(FileFixtures::file_list_response(0))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=100")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], 0);
    assert!(body["data"]["fileList"].as_array().unwrap().is_empty());
}

/// 测试分页查询 (带 lastFileId)
#[actix_web::test]
async fn test_file_lists_query_pagination() {
    // Arrange
    let ctx = TestContext::new().await;

    // 模拟分页响应，需要匹配 lastFileId 查询参数
    Mock::given(method("GET"))
        .and(path("/api/v2/file/list"))
        .and(query_param("lastFileId", "1000"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(FileFixtures::file_list_response(10))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 带 lastFileId 参数的分页请求
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=10&last_file_id=1000")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert!(resp.status().is_success());
}

/// 测试搜索功能
#[actix_web::test]
async fn test_file_lists_query_with_search() {
    // Arrange
    let ctx = TestContext::new().await;

    // 模拟搜索响应
    Mock::given(method("GET"))
        .and(path("/api/v2/file/list"))
        .and(query_param("searchData", "test.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(FileFixtures::file_list_response(1))
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act - 带搜索参数
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=100&search_data=test.txt")
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

/// 测试缺少 parentFileId 参数
#[actix_web::test]
async fn test_file_lists_query_missing_parent_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少必需的 parentFileId 参数
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?limit=100")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 Bad Request
    assert_eq!(resp.status().as_u16(), 400);
}

/// 测试缺少 limit 参数
#[actix_web::test]
async fn test_file_lists_query_missing_limit() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 缺少必需的 limit 参数
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 Bad Request
    assert_eq!(resp.status().as_u16(), 400);
}

/// 测试无效的 parentFileId 参数
#[actix_web::test]
async fn test_file_lists_query_invalid_parent_file_id() {
    // Arrange
    let ctx = TestContext::new().await;
    let app = ctx.service().await;

    // Act - 无效的 parentFileId
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=abc&limit=100")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert - 应该返回 400 Bad Request
    assert_eq!(resp.status().as_u16(), 400);
}

/// 测试服务器内部错误
#[actix_web::test]
async fn test_file_lists_query_server_error() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::internal_error("/api/v2/file/list")
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=100")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status().as_u16(), 500);
}

// ============================================================================
// 响应结构验证测试
// ============================================================================

/// 验证响应结构包含所有必需字段
#[actix_web::test]
async fn test_file_lists_query_response_structure() {
    // Arrange
    let ctx = TestContext::new().await;

    NetdiskMock::file_list(FileFixtures::file_list_response(2))
        .mount(&ctx.mock_server)
        .await;

    let app = ctx.service().await;

    // Act
    let req = test::TestRequest::get()
        .uri("/file/file_lists_query?parentFileId=0&limit=100")
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
    assert!(data.get("lastFileId").is_some(), "Data should have 'lastFileId'");
    assert!(data.get("fileList").is_some(), "Data should have 'fileList'");

    // 验证 fileList 中的文件项结构
    let file_list = data["fileList"].as_array().unwrap();
    assert!(!file_list.is_empty());

    let first_file = &file_list[0];
    assert!(first_file.get("fileId").is_some(), "File should have 'fileId'");
    assert!(first_file.get("filename").is_some(), "File should have 'filename'");
    assert!(first_file.get("type").is_some(), "File should have 'type'");
    assert!(first_file.get("size").is_some(), "File should have 'size'");
}

/// 测试不同的 limit 值
#[actix_web::test]
async fn test_file_lists_query_different_limits() {
    let test_cases = vec![
        (1, "最小 limit"),
        (10, "常用 limit"),
        (100, "最大推荐 limit"),
    ];

    for (limit, description) in test_cases {
        // Arrange
        let ctx = TestContext::new().await;

        NetdiskMock::file_list(FileFixtures::file_list_response(limit as usize))
            .mount(&ctx.mock_server)
            .await;

        let app = ctx.service().await;

        // Act
        let req = test::TestRequest::get()
            .uri(&format!("/file/file_lists_query?parentFileId=0&limit={}", limit))
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
