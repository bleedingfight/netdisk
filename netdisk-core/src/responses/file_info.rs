use super::base_config::*;
use crate::netdisk_api::prelude::*;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Serialize, Serializer};

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

// 文件信息结构体
#[derive(Debug, Deserialize)]
struct FileDetailQuery {
    #[serde(rename = "fileID")] // 确保字段名与 URL 参数名匹配
    file_id: u64,
    access_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // 关键！将 Rust 的 snake_case 映射到 JSON 的 camelCase
pub struct FileItem {
    // 整数类型
    pub file_id: i64,
    #[serde(rename = "parentFileId")]
    pub parent_file_id: u64,
    pub r#type: u8, // 使用 r#type 避免与 Rust 关键字冲突
    pub size: u64,
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
    pub search_data: Option<String>,

    // 4. searchMode (number, 选填)
    pub search_mode: Option<u8>,

    pub last_file_id: Option<i64>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListBody {
    pub last_file_id: i32,
    pub file_list: Vec<FileItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileQuery {
    #[serde(rename = "fileID")]
    #[serde(alias = "fileId")]
    pub file_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesQuery {
    #[serde(rename = "fileIds", deserialize_with = "limit_deserializer::limit_vec")]
    pub file_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    #[serde(rename = "fileID")]
    pub file_id: u64,

    #[serde(rename = "parentFileID")]
    pub parent_file_id: u64,
    pub filename: String,

    #[serde(rename = "type")]
    pub file_type: i32,
    pub size: u64,
    pub etag: String,
    pub status: i32,
    #[serde(with = "standard_format")]
    pub create_at: DateTime<Local>,
    pub trashed: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub file_id: u64,
    pub filename: String,
    pub parent_file_id: u64,
    pub r#type: i32,
    pub etag: String,
    pub size: u64,
    pub category: i32,
    pub status: i32,
    pub punish_flag: i32,
    pub s3_key_flag: String,
    pub storage_node: String,
    pub trashed: u8,

    #[serde(with = "standard_format")]
    pub create_at: DateTime<Local>,
    #[serde(with = "standard_format")]
    pub update_at: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesInfoData {
    pub fileList: Vec<FileInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryItem {
    pub name: String,
    pub parentID: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInfo {
    pub dirID: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMoveInfo {
    // #[rename = "fileIDs"]
    pub fileIDs: Vec<u64>,
    pub toParentFileID: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] // 确保字段名与API返回的 camelCase 匹配
pub struct VipInfo {
    pub vip_level: u32,

    pub vip_label: String,
    #[serde(with = "standard_format")]
    pub start_time: DateTime<Local>,

    #[serde(with = "standard_format")]
    pub end_time: chrono::DateTime<Local>, // 或使用
}

/// 开发者信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperInfo {
    #[serde(
        deserialize_with = "standard_format::deserialize_option",
        serialize_with = "standard_format::serialize_option"
    )]
    pub start_time: Option<DateTime<Local>>,

    #[serde(
        deserialize_with = "standard_format::deserialize_option",
        serialize_with = "standard_format::serialize_option"
    )]
    pub end_time: Option<DateTime<Local>>, // 或使用 chrono::DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] // 确保所有字段名与API返回的 camelCase 匹配
pub struct UserInfo {
    pub uid: u64, // 使用 u64 或 i64 (取决于 number 的实际范围)
    pub nickname: String,

    pub head_image: String,
    pub passport: String,
    pub mail: String,
    pub space_used: u64,
    pub space_permanent: u64,
    pub space_temp: u64,
    /// 临时空间到期日 (通常是字符串)
    pub space_temp_expr: u64,
    pub vip: bool,

    /// 剩余直链流量
    pub direct_traffic: u64,

    /// 直链链接是否隐藏UID
    #[serde(rename = "isHideUID")]
    pub is_hide_uid: bool,

    /// https数量
    pub https_count: u32,

    pub vip_info: Option<Vec<VipInfo>>,
    pub developer_info: Option<DeveloperInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchItem {
    #[serde(rename = "parentFileId")]
    pub parent_file_id: u64,
    pub limit: u8,
    pub search_data: Option<String>,
    pub search_mode: Option<String>,
    pub last_file_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchedData {
    pub last_file_id: i64,
    pub file_list: Vec<FileItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadUrlData {
    pub download_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // 关键！将 Rust 的 snake_case 映射到 JSON 的 camelCase
pub struct UploadFileItem {
    #[serde(alias = "parentFileID")]
    pub parent_file_id: u64,
    pub filename: String,
    pub etag: String,
    pub size: u64,
    pub duplicate: Option<u8>,
    pub contain_dir: Option<bool>,
}

/// 创建文件接口的返回内容
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileData {
    #[serde(alias = "fileID")]
    pub file_id: Option<u64>,
    pub reuse: bool,
    #[serde(alias = "preuploadID")]
    pub preupload_id: String,
    pub slice_size: u64,
    pub servers: Vec<String>,
}
impl UploadFileData {
    pub fn new(
        file_id: Option<u64>,
        reuse: bool,
        preupload_id: String,
        slice_size: u64,
        servers: Vec<String>,
    ) -> Self {
        UploadFileData {
            file_id: file_id,
            reuse: reuse,
            preupload_id: preupload_id,
            slice_size: slice_size,
            servers: servers,
        }
    }
}

pub type AccessTokenResponse = ApiResponse<AccessToken>;
pub type FileListResponse = ApiResponse<FileListBody>;
pub type FileResponse = ApiResponse<FileData>;
pub type FilesInfoResponse = ApiResponse<FilesInfoData>;
pub type PathInfoResponse = ApiResponse<EntryInfo>;
pub type UserInfoResponse = ApiResponse<UserInfo>;
pub type FileSearchResponse = ApiResponse<FileSearchedData>;
pub type DownloadUrlResponse = ApiResponse<DownloadUrlData>;
pub type UploadFileResponse = ApiResponse<UploadFileData>;
