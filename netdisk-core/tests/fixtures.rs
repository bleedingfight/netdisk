//! 测试数据工厂 (Fixtures)
//!
//! 提供测试所需的模拟数据生成器

use chrono::{DateTime, Local, Utc};
use serde_json::json;

/// 文件相关的测试数据工厂
pub struct FileFixtures;

impl FileFixtures {
    /// 创建成功的文件查询响应
    pub fn file_query_success_response(file_id: u64) -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "fileID": file_id,
                "parentFileID": 0,
                "filename": "test_file.txt",
                "type": 1,
                "size": 1024,
                "etag": "abc123def456",
                "status": 1,
                "createAt": "2024-01-01 12:00:00",
                "trashed": 0
            },
            "x-traceID": "trace-test-12345"
        })
    }

    /// 创建文件不存在的错误响应
    pub fn file_not_found_response() -> serde_json::Value {
        json!({
            "code": 404,
            "message": "file not found",
            "data": null,
            "x-traceID": "trace-test-error-404"
        })
    }

    /// 创建认证失败的错误响应
    pub fn auth_failed_response() -> serde_json::Value {
        json!({
            "code": 401,
            "message": "unauthorized",
            "data": null,
            "x-traceID": "trace-test-error-401"
        })
    }

    /// 创建文件列表响应
    pub fn file_list_response(count: usize) -> serde_json::Value {
        let file_list: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                json!({
                    "fileId": 1000 + i as u64,
                    "parentFileId": 0,
                    "type": 1,
                    "size": 1024 * (i + 1) as u64,
                    "category": 1,
                    "status": 1,
                    "punishFlag": 0,
                    "trashed": 0,
                    "filename": format!("file_{}.txt", i),
                    "etag": format!("etag_{}", i),
                    "createAt": "2024-01-01 12:00:00",
                    "updateAt": "2024-01-01 12:00:00"
                })
            })
            .collect();

        json!({
            "code": 0,
            "message": "success",
            "data": {
                "lastFileId": if count > 0 { 1000 + count as i32 - 1 } else { 0 },
                "fileList": file_list
            },
            "x-traceID": "trace-test-list-12345"
        })
    }

    /// 创建批量文件信息响应
    pub fn files_info_response(file_ids: &[u64]) -> serde_json::Value {
        let file_list: Vec<serde_json::Value> = file_ids
            .iter()
            .map(|&id| {
                json!({
                    "fileId": id,
                    "filename": format!("file_{}.txt", id),
                    "parentFileId": 0,
                    "type": 1,
                    "etag": format!("etag_{}", id),
                    "size": 1024,
                    "category": 1,
                    "status": 1,
                    "punishFlag": 0,
                    "s3KeyFlag": "",
                    "storageNode": "node1",
                    "trashed": 0,
                    "createAt": "2024-01-01 12:00:00",
                    "updateAt": "2024-01-01 12:00:00"
                })
            })
            .collect();

        json!({
            "code": 0,
            "message": "success",
            "data": {
                "fileList": file_list
            },
            "x-traceID": "trace-test-infos-12345"
        })
    }

    /// 创建下载链接响应
    pub fn download_url_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "downloadUrl": "https://download.example.com/file/abc123?token=xyz"
            },
            "x-traceID": "trace-test-download-12345"
        })
    }

    /// 创建创建目录响应
    pub fn mkdir_response(dir_id: u64) -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "dirID": dir_id
            },
            "x-traceID": "trace-test-mkdir-12345"
        })
    }

    /// 创建文件移动成功响应
    pub fn file_move_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": null,
            "x-traceID": "trace-test-move-12345"
        })
    }

    /// 创建移动到回收站成功响应
    pub fn trash_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": null,
            "x-traceID": "trace-test-trash-12345"
        })
    }

    /// 创建永久删除成功响应
    pub fn delete_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": null,
            "x-traceID": "trace-test-delete-12345"
        })
    }
}

/// Token 相关的测试数据工厂
pub struct TokenFixtures;

impl TokenFixtures {
    /// 创建有效的 access token 响应
    pub fn valid_token_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "accessToken": "valid_access_token_12345",
                "expiredAt": "2099-12-31T23:59:59Z"
            },
            "x-traceID": "trace-test-token-12345"
        })
    }

    /// 创建过期的 token 响应
    pub fn expired_token_response() -> serde_json::Value {
        json!({
            "code": 401,
            "message": "token expired",
            "data": null,
            "x-traceID": "trace-test-token-expired"
        })
    }

    /// 创建无效凭证响应
    pub fn invalid_credentials_response() -> serde_json::Value {
        json!({
            "code": 401,
            "message": "invalid client_id or client_secret",
            "data": null,
            "x-traceID": "trace-test-invalid-creds"
        })
    }
}

/// 用户相关的测试数据工厂
pub struct UserFixtures;

impl UserFixtures {
    /// 创建用户信息响应
    pub fn user_info_response() -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "uid": 123456,
                "nickname": "TestUser",
                "headImage": "https://example.com/avatar.png",
                "passport": "test@example.com",
                "mail": "test@example.com",
                "spaceUsed": 1073741824_u64,
                "spacePermanent": 10737418240_u64,
                "spaceTemp": 0,
                "spaceTempExpr": 0,
                "vip": true,
                "directTraffic": 10737418240_u64,
                "isHideUID": false,
                "httpsCount": 100,
                "vipInfo": [{
                    "vipLevel": 1,
                    "vipLabel": "VIP",
                    "startTime": "2024-01-01 00:00:00",
                    "endTime": "2025-01-01 00:00:00"
                }],
                "developerInfo": null
            },
            "x-traceID": "trace-test-user-12345"
        })
    }
}

/// 分享相关的测试数据工厂
pub struct ShareFixtures;

impl ShareFixtures {
    /// 创建分享列表响应
    pub fn share_list_response(count: usize) -> serde_json::Value {
        let share_list: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                json!({
                    "shareId": 100 + i as i64,
                    "shareKey": format!("key_{}", i),
                    "shareName": format!("Share {}", i),
                    "expiration": "2099-12-31 23:59:59",
                    "expired": 0,
                    "sharePwd": "1234",
                    "trafficSwitch": 1,
                    "trafficLimitSwitch": 0,
                    "trafficLimit": 10737418240_u64,
                    "bytesCharge": 0,
                    "previewCount": 0,
                    "downloadCount": 0,
                    "saveCount": 0
                })
            })
            .collect();

        json!({
            "code": 0,
            "message": "success",
            "data": {
                "lastShareId": if count > 0 { 100 + count as u64 - 1 } else { 0 },
                "shareList": share_list
            },
            "x-traceID": "trace-test-share-list-12345"
        })
    }

    /// 创建分享创建成功响应
    pub fn share_create_response(share_id: u64) -> serde_json::Value {
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "shareID": share_id,
                "shareKey": "abc123"
            },
            "x-traceID": "trace-test-share-create-12345"
        })
    }
}
