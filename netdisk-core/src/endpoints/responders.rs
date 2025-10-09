use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fs;
use std::path::PathBuf;

/// 授权信息
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    client_id: String,
    client_secret: String,
}

impl AuthConfig {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    pub fn new(c_id: String, c_sec: String) -> Self {
        AuthConfig {
            client_id: c_id,
            client_secret: c_sec,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            client_id: "123".to_string(),
            client_secret: "123".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConfig<T> {
    #[serde(default)]
    pub cache: bool,
    pub data: T,
}

impl<T> RequestConfig<T> {
    pub fn new(cache: bool, data: T) -> Self {
        RequestConfig {
            cache: cache,
            data: data,
        }
    }
}

/// 序列化配置文件
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    client_id: String,
    client_secret: String,
    server: Option<PlatformConfig>, // 可选字段
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PlatformConfig {
    platform_domain: String,
    platform: String,
}

/// 返回授权信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessToken {
    pub access_token: String,
    pub expired_at: DateTime<Utc>,
}
impl AccessToken {
    pub fn new(access_token: String, expired_at: DateTime<Utc>) -> Self {
        AccessToken {
            access_token: access_token,
            expired_at: expired_at,
        }
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        AccessToken {
            access_token: "123".to_string(),
            expired_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
    #[serde(rename = "x-traceID")]
    pub x_trace_id: String,
}
impl<T> ApiResponse<T> {
    pub fn new(code: i32, message: String, data: T, x_trace_id: String) -> Self {
        ApiResponse {
            code: code,
            message: message,
            data: data,
            x_trace_id: x_trace_id,
        }
    }
}

impl Default for PlatformConfig {
    fn default() -> Self {
        PlatformConfig {
            platform_domain: "open-api.123pan.com".to_string(),
            platform: "open_platform".to_string(),
        }
    }
}
impl PlatformConfig {
    pub fn platform_domain(&self) -> &str {
        &self.platform_domain
    }
    pub fn platform(&self) -> &str {
        &self.platform
    }
}
impl Config {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    pub fn new(c_id: String, c_sec: String, service: Option<PlatformConfig>) -> Self {
        Config {
            client_id: c_id,
            client_secret: c_sec,
            server: service,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.client_id.trim().is_empty() && !self.client_secret.trim().is_empty()
    }

    /// 从文件解析配置
    pub fn from_file(path: &PathBuf) -> Option<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            let conf: Config = toml::from_str(&content).ok()?;
            if conf.is_valid() {
                Some(conf)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 从环境变量解析配置
    fn from_env() -> Option<Self> {
        let client_id = env::var("NETDISK_CLIENT_ID").ok()?;
        let client_secret = env::var("NETDISK_CLIENT_SECRET").ok()?;
        let conf = Config {
            client_id,
            client_secret,
            server: Some(PlatformConfig::default()),
        };
        if conf.is_valid() {
            Some(conf)
        } else {
            None
        }
    }
    pub fn load() -> Result<Self, String> {
        // 1. ~/.config/netdisk/config.toml
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".config/netdisk/config.toml");
            if let Some(conf) = Config::from_file(&config_path) {
                return Ok(conf);
            }
        }

        // 2. ./config.toml
        let cwd = env::current_dir().unwrap_or_default();
        let local_path = cwd.join("config.toml");
        if let Some(conf) = Config::from_file(&local_path) {
            return Ok(conf);
        }

        // 3. 环境变量
        if let Some(conf) = Config::from_env() {
            return Ok(conf);
        }

        // 4. 全部失败
        Err("无法找到合法的配置: 请检查 ~/.config/netdisk/config.toml, ./config.toml, 或设置 NETDISK_CLIENT_ID / NETDISK_CLIENT_SECRET".to_string())
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            client_id: "123".to_string(),
            client_secret: "123".to_string(),
            server: Some(PlatformConfig::default()),
        }
    }
}

// impl<T: serde::Serialize> Responder for ApiResponse<T> {
//     type Body = BoxBody;

//     fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
//         let body = serde_json::to_string(&self).unwrap();

//         // Create response and set content type
//         HttpResponse::Ok()
//             .content_type(ContentType::json())
//             .body(body)
//     }
// }

impl<T: serde::Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        // 尝试将结构体序列化为 JSON 字符串
        match serde_json::to_string(&self) {
            Ok(body) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body),
            Err(e) => HttpResponse::InternalServerError().body(format!("序列化错误: {}", e)),
        }
    }
}

/// 创建文件接口的返回内容
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFile {
    pub file_id: i64,

    // 布尔类型
    pub reuse: bool,

    // 字符串类型
    pub preupload_id: String,

    // 整数类型（通常 slice size 较大，用 i64 或 u64 更安全）
    pub slice_size: u64,

    // 字符串数组（List of Strings）
    pub servers: Vec<String>,
}
impl CreateFile {
    pub fn new(
        file_id: i64,
        reuse: bool,
        preupload_id: String,
        slice_size: u64,
        servers: Vec<String>,
    ) -> Self {
        CreateFile {
            file_id: file_id,
            reuse: reuse,
            preupload_id: preupload_id,
            slice_size: slice_size,
            servers: servers,
        }
    }
}

// 文件信息结构体
#[derive(Debug, Deserialize)]
struct FileDetailQuery {
    #[serde(rename = "fileID")] // 确保字段名与 URL 参数名匹配
    file_id: u64,
    access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")] // 关键！将 Rust 的 snake_case 映射到 JSON 的 camelCase
pub struct FileItem {
    // 整数类型
    pub file_id: i64,
    pub parent_file_id: i64,
    pub r#type: u8, // 使用 r#type 避免与 Rust 关键字冲突
    pub size: i64,
    pub category: u8,
    pub status: u8,
    pub punish_flag: u8,
    pub trashed: u8,

    // 字符串类型
    pub filename: String,
    pub etag: String,
    // pub s3_key_flag: String,
    // pub storage_node: String,

    // 日期时间字符串（需要 chrono 解析，注意 JSON 格式 "2025-09-24 14:46:51"）
    // Actix/Serde 默认不支持这种带空格的格式，需要特殊处理
    // 1. 最简单：直接使用 String
    // pub create_at: String,
    // pub update_at: String,

    // 2. 推荐：使用 String 但在业务逻辑中解析，或使用自定义 serde 模块
    #[serde(with = "standard_format")]
    pub create_at: DateTime<Local>,
    #[serde(with = "standard_format")]
    pub update_at: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    // 1. parentFileId (number, 必填)
    // 根目录传 0。这里假设 API 接受 i64 或 u64
    #[serde(rename = "parentFileId")]
    pub parent_file_id: i64,

    // 2. limit (number, 必填)
    // 这里的 limit 对应 JSON 中的 limit，如果查询参数是小驼峰，则不需要 rename。
    // 如果 URL 参数是小驼峰 (limit)，Rust 字段是 snake_case (limit_)，则需要 rename。
    // 假设 URL 参数就是 limit
    pub limit: u8,

    // 3. searchData (string, 选填)
    // 使用 Option<String> 允许该参数缺失或为空
    #[serde(rename = "searchData")]
    pub search_data: Option<String>,

    // 4. searchMode (number, 选填)
    // 使用 Option<u8> 允许该参数缺失
    #[serde(rename = "searchMode")]
    pub search_mode: Option<u8>,

    // 5. lastFileId (number, 选填)
    #[serde(rename = "lastFileId")]
    pub last_file_id: Option<i64>,
}

mod standard_format {
    // 将顶层 use 引入到模块内部作用域
    use super::{DateTime, Deserialize, Deserializer, Local, NaiveDateTime, Serializer, TimeZone};

    // API 要求的日期时间格式
    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    // --- 1. 反序列化 (JSON String -> Rust DateTime<Local>) ---

    // 注意：这里的 'de 必须存在，以满足 Deserializer Trait 的要求
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match NaiveDateTime::parse_from_str(&s, FORMAT) {
            Ok(dt) => {
                // 将 NaiveDateTime 转换为 Local 时区下的 DateTime
                // unwrap() 在这里通常是安全的，因为 NaiveDateTime 是有效的
                Ok(Local.from_local_datetime(&dt).unwrap())
            }
            Err(_) => Err(serde::de::Error::custom(format!("无效日期格式：{}", s))),
        }
    }

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 1. 使用 chrono::format 将 DateTime 转换为所需的字符串格式
        let s = format!("{}", date.format(FORMAT));

        // 2. 将字符串交给 Serializer 进行输出
        serializer.serialize_str(&s)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListBody {
    pub last_file_id: i32,
    pub file_list: Vec<FileItem>,
}

// impl<T: serde::Serialize> Responder for ApiResponse<T> {
//     type Body = BoxBody;

//     fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
//         // 尝试将结构体序列化为 JSON 字符串
//         match serde_json::to_string(&self) {
//             Ok(body) => HttpResponse::Ok()
//                 .content_type(ContentType::json())
//                 .body(body),
//             Err(e) => {
//                 // 如果序列化失败，返回 500 错误
//                 HttpResponse::InternalServerError().body(format!("序列化错误: {}", e))
//             }
//         }
//     }
// }
pub type AccessTokenResponse = ApiResponse<AccessToken>;
pub type CreateFileResponse = ApiResponse<CreateFile>;
pub type FileListResponse = ApiResponse<FileListBody>;
