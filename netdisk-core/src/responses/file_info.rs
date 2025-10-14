use super::base_config::*;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

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

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    #[serde(rename = "fileID")]
    pub file_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct FilesQuery {
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    /// 文件ID
    #[serde(rename = "fileID")]
    pub file_id: u64,

    #[serde(rename = "parentFileID")]
    pub parent_file_id: u64,

    pub filename: String,

    /// 文件类型
    #[serde(rename = "type")]
    pub file_type: i32,

    /// 文件大小（字节）
    pub size: u64,

    /// 文件的ETag标识
    pub etag: String,

    /// 文件状态
    pub status: i32,

    #[serde(with = "standard_format")]
    pub create_at: DateTime<Local>,

    /// 是否被删除（0表示未删除，1表示已删除）
    pub trashed: i32,
}

pub type AccessTokenResponse = ApiResponse<AccessToken>;
pub type CreateFileResponse = ApiResponse<CreateFile>;
pub type FileListResponse = ApiResponse<FileListBody>;
pub type FileResponse = ApiResponse<FileData>;
