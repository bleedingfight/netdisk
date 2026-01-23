//! Mock Server 模块
//!
//! 提供 wiremock 的便捷封装，用于模拟 123云盘 API

use wiremock::matchers::{method, path, query_param, header};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// 123云盘 API Mock 构建器
pub struct NetdiskMock;

impl NetdiskMock {
    /// 模拟文件详情查询 API
    /// 
    /// # Arguments
    /// * `file_id` - 要查询的文件 ID
    /// * `response` - 返回的 JSON 响应
    /// 
    /// # Example
    /// ```ignore
    /// let mock_server = MockServer::start().await;
    /// NetdiskMock::file_query(12345, FileFixtures::file_query_success_response(12345))
    ///     .mount(&mock_server)
    ///     .await;
    /// ```
    pub fn file_query(file_id: i64, response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v1/file/detail"))
            .and(query_param("fileID", file_id.to_string()))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟文件详情查询 - 任意文件 ID
    pub fn file_query_any(response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v1/file/detail"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟文件列表查询 API
    pub fn file_list(response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v2/file/list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟批量文件信息查询 API
    pub fn files_info(response: serde_json::Value) -> Mock {
        Mock::given(method("POST"))
            .and(path("/api/v1/file/infos"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟文件下载链接 API
    pub fn download(file_id: i64, response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v1/file/download_info"))
            .and(query_param("fileID", file_id.to_string()))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟创建目录 API
    pub fn mkdir(response: serde_json::Value) -> Mock {
        Mock::given(method("POST"))
            .and(path("/upload/v1/file/mkdir"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟用户信息 API
    pub fn user_info(response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v1/user/info"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟获取 access token API
    pub fn access_token(response: serde_json::Value) -> Mock {
        // 不匹配 method，因为 wiremock 对 method 匹配较严格
        // 只匹配 path 即可
        Mock::given(path("/api/v1/access_token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟文件移动 API
    pub fn file_move(response: serde_json::Value) -> Mock {
        Mock::given(path("/api/v1/file/move"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟移动到回收站 API
    pub fn file_trash(response: serde_json::Value) -> Mock {
        Mock::given(path("/api/v1/file/trash"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟永久删除 API
    pub fn file_delete(response: serde_json::Value) -> Mock {
        Mock::given(path("/api/v1/file/delete"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟分享列表 API
    pub fn share_list(response: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/api/v1/share/list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟创建分享 API
    pub fn share_create(response: serde_json::Value) -> Mock {
        Mock::given(method("POST"))
            .and(path("/api/v1/share/create"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟 API 错误响应
    pub fn error_response(api_path: &str, http_method: &str, status_code: u16, response: serde_json::Value) -> Mock {
        let method_matcher = match http_method.to_uppercase().as_str() {
            "GET" => method("GET"),
            "POST" => method("POST"),
            "PUT" => method("PUT"),
            "DELETE" => method("DELETE"),
            _ => method("GET"),
        };

        Mock::given(method_matcher)
            .and(path(api_path))
            .respond_with(
                ResponseTemplate::new(status_code)
                    .set_body_json(response)
                    .insert_header("Content-Type", "application/json"),
            )
    }

    /// 模拟网络超时
    pub fn timeout(api_path: &str) -> Mock {
        Mock::given(path(api_path))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_secs(60)),
            )
    }

    /// 模拟服务器内部错误
    pub fn internal_error(api_path: &str) -> Mock {
        Mock::given(path(api_path))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
    }
}

/// Mock 验证辅助
pub struct MockVerify;

impl MockVerify {
    /// 验证 mock 被调用了指定次数
    pub async fn called_times(mock_server: &MockServer, expected: u64) {
        let received = mock_server.received_requests().await.unwrap_or_default();
        assert_eq!(
            received.len() as u64,
            expected,
            "Expected {} requests, but received {}",
            expected,
            received.len()
        );
    }

    /// 验证 mock 至少被调用一次
    pub async fn called_at_least_once(mock_server: &MockServer) {
        let received = mock_server.received_requests().await.unwrap_or_default();
        assert!(
            !received.is_empty(),
            "Expected at least one request, but received none"
        );
    }
}
