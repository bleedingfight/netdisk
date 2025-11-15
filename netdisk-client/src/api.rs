use crate::client::NetdiskClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Authentication API
impl NetdiskClient {
    /// Get access token using client credentials
    pub async fn get_access_token(&mut self) -> Result<AccessTokenResponse> {
        let request = AuthRequest {
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
        };

        let response: AccessTokenResponse = self.post("/access_token", &request).await?;
        self.set_access_token(response.access_token.clone());
        Ok(response)
    }
}

/// File management API
impl NetdiskClient {
    /// Get file list
    pub async fn get_file_list(&self, query: FileListQuery) -> Result<FileListResponse> {
        let mut params = vec![
            ("parentFileId", query.parent_file_id.to_string()),
            ("limit", query.limit.to_string()),
        ];

        if let Some(search_data) = &query.search_data {
            params.push(("searchData", search_data.clone()));
        }
        if let Some(search_mode) = query.search_mode {
            params.push(("searchMode", search_mode.to_string()));
        }
        if let Some(last_file_id) = query.last_file_id {
            params.push(("lastFileId", last_file_id.to_string()));
        }

        let path = format!("/api/v2/file/list?{}", 
            params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        );
        
        self.get(&path).await
    }

    /// Get single file information
    pub async fn get_file_info(&self, file_id: i64) -> Result<FileResponse> {
        let path = format!("/api/v1/file/detail?fileID={}", file_id);
        self.get(&path).await
    }

    /// Get multiple files information
    pub async fn get_files_info(&self, file_ids: Vec<u64>) -> Result<FilesInfoResponse> {
        let request = FilesQuery { file_ids };
        self.post("/api/v1/file/infos", &request).await
    }

    /// Create directory
    pub async fn create_directory(&self, name: String, parent_id: u64) -> Result<PathInfoResponse> {
        let request = EntryItem { name, parent_id };
        self.post("/upload/v1/file/mkdir", &request).await
    }

    /// Move files to specific directory
    pub async fn move_files(&self, file_ids: Vec<u64>, to_parent_file_id: u64) -> Result<ApiResponse<()>> {
        let request = FileMoveInfo {
            file_ids,
            to_parent_file_id,
        };
        self.post("/api/v1/file/move", &request).await
    }

    /// Move files to trash
    pub async fn move_to_trash(&self, file_ids: Vec<u64>) -> Result<ApiResponse<()>> {
        let request = FileIdsRequest { file_ids };
        self.post("/api/v1/file/trash", &request).await
    }

    /// Delete files permanently
    pub async fn delete_files(&self, file_ids: Vec<u64>) -> Result<ApiResponse<()>> {
        let request = FileIdsRequest { file_ids };
        self.post("/api/v1/file/delete", &request).await
    }

    /// Get file download information
    pub async fn get_download_info(&self, file_id: i64) -> Result<DownloadUrlResponse> {
        let path = format!("/api/v1/file/download_info?fileId={}", file_id);
        self.get(&path).await
    }
}

/// Share management API
impl NetdiskClient {
    /// Get share list
    pub async fn get_share_list(&self, limit: u8, last_share_id: Option<i64>) -> Result<SharedListDataResponse> {
        let mut params = vec![("limit", limit.to_string())];
        if let Some(last_id) = last_share_id {
            params.push(("lastShareId", last_id.to_string()));
        }

        let path = format!("/api/v1/share/list?{}", 
            params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        );
        
        self.get(&path).await
    }

    /// Create file share link
    pub async fn create_share(&self, request: ShareItem) -> Result<SharedDataResponse> {
        self.post("/api/v1/share/create", &request).await
    }

    /// Get payment share list
    pub async fn get_payment_list(&self, limit: u8, last_share_id: Option<i64>) -> Result<PayShareDataResponse> {
        let mut params = vec![("limit", limit.to_string())];
        if let Some(last_id) = last_share_id {
            params.push(("lastShareId", last_id.to_string()));
        }

        let path = format!("/api/v1/share/payment/list?{}", 
            params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        );
        
        self.get(&path).await
    }

    /// Set share link parameters
    pub async fn set_share_info(&self, request: ShareLinkItem) -> Result<ApiResponse<()>> {
        self.put("/api/v1/share/list/info", &request).await
    }

    /// Create paid share link
    pub async fn create_paid_share(&self, request: PayLinkItem) -> Result<SharedDataResponse> {
        self.post("/api/v1/share/content-payment/create", &request).await
    }

    /// Modify paid share link info
    pub async fn modify_paid_share_info(&self, request: ShareLinkItem) -> Result<ApiResponse<()>> {
        self.put("/api/v1/share/list/payment/info", &request).await
    }
}

/// User information API
impl NetdiskClient {
    /// Get user information
    pub async fn get_user_info(&self) -> Result<UserInfoResponse> {
        self.get("/api/v1/user/info").await
    }
}

/// Upload API
impl NetdiskClient {
    /// Upload file (create upload session)
    pub async fn upload_file(&self, request: UploadFileItem) -> Result<UploadFileResponse> {
        self.post("/upload/v2/file/create", &request).await
    }
}

// Request and Response types

#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expired_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListQuery {
    pub parent_file_id: i64,
    pub limit: u8,
    pub search_data: Option<String>,
    pub search_mode: Option<u8>,
    pub last_file_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilesQuery {
    pub file_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryItem {
    pub name: String,
    #[serde(rename = "parentID")]
    pub parent_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMoveInfo {
    #[serde(rename = "fileIDs")]
    pub file_ids: Vec<u64>,
    #[serde(rename = "toParentFileID")]
    pub to_parent_file_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileIdsRequest {
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareItem {
    pub share_name: String,
    pub share_expire: u8,
    #[serde(rename = "fileIDList")]
    pub file_id_list: String,
    pub share_pwd: Option<String>,
    pub traffic_switch: Option<u8>,
    pub traffic_limit_switch: Option<u8>,
    pub traffic_limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareLinkItem {
    pub share_id_list: Vec<u64>,
    pub traffic_switch: Option<i32>,
    pub traffic_limit_switch: Option<i32>,
    pub traffic_limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayLinkItem {
    pub share_name: String,
    #[serde(rename = "fileIDList")]
    pub file_id_list: String,
    pub pay_amount: u32,
    pub is_reward: Option<u8>,
    pub resource_desc: Option<String>,
    pub traffic_switch: Option<u8>,
    pub traffic_limit_switch: Option<u8>,
    pub traffic_limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileItem {
    #[serde(alias = "parentFileID")]
    pub parent_file_id: u64,
    pub filename: String,
    pub etag: String,
    pub size: u64,
    pub duplicate: Option<u8>,
    pub contain_dir: Option<bool>,
}

// Response type aliases
pub type ApiResponse<T> = crate::client::ApiResponse<T>;
pub type FileListResponse = ApiResponse<FileListBody>;
pub type FileResponse = ApiResponse<FileData>;
pub type FilesInfoResponse = ApiResponse<FilesInfoData>;
pub type PathInfoResponse = ApiResponse<EntryInfo>;
pub type DownloadUrlResponse = ApiResponse<DownloadUrlData>;
pub type UserInfoResponse = ApiResponse<UserInfo>;
pub type SharedDataResponse = ApiResponse<SharedData>;
pub type SharedListDataResponse = ApiResponse<ShareListData>;
pub type PayShareDataResponse = ApiResponse<PayListItem>;
pub type UploadFileResponse = ApiResponse<UploadFileData>;

// Data structures for responses
#[derive(Debug, Serialize, Deserialize)]
pub struct FileListBody {
    pub last_file_id: i32,
    pub file_list: Vec<FileItem>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub create_at: String,
    pub trashed: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilesInfoData {
    #[serde(rename = "fileList")]
    pub file_list: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub create_at: String,
    pub update_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryInfo {
    #[serde(rename = "dirID")]
    pub dir_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadUrlData {
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: u64,
    pub nickname: String,
    pub head_image: String,
    pub passport: String,
    pub mail: String,
    pub space_used: u64,
    pub space_permanent: u64,
    pub space_temp: u64,
    pub space_temp_expr: u64,
    pub vip: bool,
    pub direct_traffic: u64,
    #[serde(rename = "isHideUID")]
    pub is_hide_uid: bool,
    pub https_count: u32,
    pub vip_info: Option<Vec<VipInfo>>,
    pub developer_info: Option<DeveloperInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VipInfo {
    pub vip_level: u32,
    pub vip_label: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeveloperInfo {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub file_id: i64,
    #[serde(rename = "parentFileId")]
    pub parent_file_id: u64,
    pub r#type: u8,
    pub size: u64,
    pub category: u8,
    pub status: u8,
    pub punish_flag: u8,
    pub trashed: u8,
    pub filename: String,
    pub etag: String,
    pub create_at: String,
    pub update_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SharedData {
    #[serde(rename = "shareID")]
    pub share_id: u64,
    pub share_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareListData {
    pub last_share_id: u64,
    pub share_list: Vec<ShareItemData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareItemData {
    #[serde(rename = "shareId")]
    pub share_id: i64,
    pub share_key: String,
    pub share_name: String,
    pub expiration: String,
    pub expired: u8,
    pub share_pwd: String,
    pub traffic_switch: u8,
    pub traffic_limit_switch: u8,
    pub traffic_limit: u64,
    pub bytes_charge: u64,
    pub preview_count: u32,
    pub download_count: u32,
    pub save_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayListItem {
    pub last_share_id: i8,
    pub share_list: Option<Vec<PayShareItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayShareItem {
    pub share_id: i64,
    pub share_key: String,
    pub share_name: String,
    pub pay_amount: u32,
    pub amount: i32,
    pub expiration: String,
    pub expired: u8,
    pub traffic_switch: u8,
    pub traffic_limit_switch: u8,
    pub traffic_limit: u64,
    pub bytes_charge: u64,
    pub preview_count: u32,
    pub download_count: u32,
    pub save_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileData {
    #[serde(alias = "fileID")]
    pub file_id: Option<u64>,
    pub reuse: bool,
    #[serde(alias = "preuploadID")]
    pub preupload_id: String,
    pub slice_size: u64,
    pub servers: Vec<String>,
}