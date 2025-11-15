use crate::error::{NetdiskError, Result};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the netdisk client
#[derive(Debug, Clone)]
pub struct NetdiskConfig {
    /// Base URL for the API
    pub base_url: String,
    /// Platform domain
    pub platform_domain: String,
    /// Platform identifier
    pub platform: String,
    /// Request timeout
    pub timeout: Duration,
    /// Client ID for authentication
    pub client_id: String,
    /// Client secret for authentication
    pub client_secret: String,
}

impl Default for NetdiskConfig {
    fn default() -> Self {
        Self {
            base_url: "https://open-api.123pan.com".to_string(),
            platform_domain: "open-api.123pan.com".to_string(),
            platform: "open_platform".to_string(),
            timeout: Duration::from_secs(30),
            client_id: "".to_string(),
            client_secret: "".to_string(),
        }
    }
}

/// Main client for interacting with the netdisk API
pub struct NetdiskClient {
    pub client: Client,
    pub config: NetdiskConfig,
    pub access_token: Option<String>,
}

impl NetdiskClient {
    /// Create a new client with the given configuration
    pub fn new(config: NetdiskConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            access_token: None,
        }
    }

    /// Create a new client with default configuration
    pub fn new_default() -> Self {
        Self::new(NetdiskConfig::default())
    }

    /// Set the access token for authenticated requests
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Get the current access token
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Build a request with common headers
    fn build_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.config.base_url, path)
        };

        let mut request = self.client.request(method, &url);

        // Add common headers
        request = request
            .header("Content-Type", "application/json")
            .header("Platform", &self.config.platform);

        // Add authorization header if token is available
        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
    }

    /// Send a GET request
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.send_request(self.build_request(reqwest::Method::GET, path))
            .await
    }

    /// Send a POST request with JSON body
    pub async fn post<T, U>(&self, path: &str, body: &U) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        U: Serialize + ?Sized,
    {
        let request = self.build_request(reqwest::Method::POST, path).json(body);
        self.send_request(request).await
    }

    /// Send a PUT request with JSON body
    pub async fn put<T, U>(&self, path: &str, body: &U) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        U: Serialize + ?Sized,
    {
        let request = self.build_request(reqwest::Method::PUT, path).json(body);
        self.send_request(request).await
    }

    /// Send a DELETE request
    pub async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.send_request(self.build_request(reqwest::Method::DELETE, path))
            .await
    }

    /// Send a request and handle the response
    async fn send_request<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NetdiskError::ApiError {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, body),
            });
        }

        let api_response: ApiResponse<T> = response.json().await?;

        if api_response.code != 0 {
            return Err(NetdiskError::ApiError {
                code: api_response.code,
                message: api_response.message,
            });
        }

        api_response
            .data
            .ok_or_else(|| NetdiskError::ApiError {
                code: api_response.code,
                message: "Response data is missing".to_string(),
            })
    }
}

/// Generic API response structure
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    #[serde(rename = "x-traceID")]
    pub x_trace_id: String,
}