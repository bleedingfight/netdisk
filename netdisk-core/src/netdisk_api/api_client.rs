//! API 客户端抽象层
//!
//! 提供可测试的 HTTP 客户端封装，支持依赖注入

use reqwest::Client;

/// API 客户端配置
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// API 基础 URL (例如: https://open-api.123pan.com)
    pub base_url: String,
    /// 平台标识
    pub platform: String,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://open-api.123pan.com".to_string(),
            platform: "open_platform".to_string(),
        }
    }
}

impl ApiClientConfig {
    /// 创建新的配置
    pub fn new(base_url: impl Into<String>, platform: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            platform: platform.into(),
        }
    }

    /// 用于测试的配置，指向 mock server
    pub fn for_test(mock_server_url: impl Into<String>) -> Self {
        Self {
            base_url: mock_server_url.into(),
            platform: "open_platform".to_string(),
        }
    }
}

/// API 客户端
/// 
/// 封装 HTTP 请求逻辑，支持依赖注入以便测试
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    config: ApiClientConfig,
}

impl ApiClient {
    /// 使用默认配置创建客户端
    pub fn new() -> Self {
        Self::with_config(ApiClientConfig::default())
    }

    /// 使用指定配置创建客户端
    pub fn with_config(config: ApiClientConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// 获取平台标识
    pub fn platform(&self) -> &str {
        &self.config.platform
    }

    /// 构建完整的 API URL
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }

    /// 获取内部的 reqwest Client
    pub fn http_client(&self) -> &Client {
        &self.client
    }

    /// 发送 GET 请求
    pub async fn get<T>(&self, path: &str, token: &str) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.url(path);
        let response = self.client
            .get(&url)
            .header("Platform", &self.config.platform)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(ApiError::Request)?;

        self.handle_response(response).await
    }

    /// 发送带查询参数的 GET 请求
    pub async fn get_with_query<T, Q>(&self, path: &str, query: &Q, token: &str) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize,
    {
        let url = self.url(path);
        let response = self.client
            .get(&url)
            .query(query)
            .header("Platform", &self.config.platform)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(ApiError::Request)?;

        self.handle_response(response).await
    }

    /// 发送 POST 请求
    pub async fn post<T, B>(&self, path: &str, body: &B, token: &str) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = self.url(path);
        let response = self.client
            .post(&url)
            .header("Platform", &self.config.platform)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .map_err(ApiError::Request)?;

        self.handle_response(response).await
    }

    /// 处理响应
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        response
            .json()
            .await
            .map_err(ApiError::Deserialize)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// API 错误类型
#[derive(Debug)]
pub enum ApiError {
    /// HTTP 请求错误
    Request(reqwest::Error),
    /// HTTP 状态码错误
    Http { status: u16, body: String },
    /// 反序列化错误
    Deserialize(reqwest::Error),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "请求失败: {}", e),
            ApiError::Http { status, body } => {
                write!(f, "HTTP 错误 {}: {}", status, body)
            }
            ApiError::Deserialize(e) => write!(f, "响应解析失败: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<ApiError> for actix_web::Error {
    fn from(err: ApiError) -> Self {
        actix_web::error::ErrorInternalServerError(err.to_string())
    }
}
