use super::base_config::*;
use crate::netdisk_api::prelude::*;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareItem {
    /// 必填：分享链接名称
    pub share_name: String,

    /// 必填：分享链接有效期天数（单位：天）
    /// 固定只能填写: 1、7、30、0 (0代表永久分享)
    pub share_expire: ShareExpireDays, // u8 足够存储 0, 1, 7, 30

    /// 必填：分享文件ID列表，以逗号分割。最大只支持拼接100个文件ID。示例: "1,2,3"
    #[serde(rename = "fileIDList")]
    pub file_id_list: String,

    /// 选填：设置分享链接提取码
    // 注意：如果是 None 则表示未设置
    pub share_pwd: Option<String>,

    /// 选填：分享提取流量包开关
    /// 1: 全部关闭, 2: 打开游客免登录提取, 3: 打开超流量用户提取, 4: 全部开启
    pub traffic_switch: Option<u8>,

    /// 选填：分享提取流量包流量限制开关
    /// 1: 关闭限制, 2: 打开限制
    pub traffic_limit_switch: Option<u8>,

    /// 选填：分享提取流量包限制流量（单位：字节）
    pub traffic_limit: Option<u64>, // 使用 u64 来匹配 int64 的要求，确保足够的容量
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareExpireDays {
    #[serde(rename = "1")] // 在序列化/反序列化时，将这个成员映射为数字 1
    OneDay = 1,

    #[serde(rename = "7")]
    SevenDays = 7,

    #[serde(rename = "30")]
    ThirtyDays = 30,

    // 0 代表永久分享
    #[serde(rename = "0")]
    Permanent = 0,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedData {
    #[serde(rename = "shareID")]
    pub share_id: u64,
    pub share_key: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareQuery {
    pub last_share_id: Option<i64>,
    pub limit: u8,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareItemData {
    #[serde(rename = "shareId")]
    pub share_id: i64,

    pub share_key: String,
    pub share_name: String,
    #[serde(with = "standard_format")]
    pub expiration: DateTime<Local>,

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

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareListData {
    pub last_share_id: u64,
    pub share_list: Vec<ShareItemData>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkItem {
    pub share_id_list: Vec<u64>,
    pub traffic_switch: Option<i32>,
    pub traffic_limit_switch: Option<i32>,
    pub traffic_limit: Option<i64>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayShareItem {
    pub share_id: i64,
    pub share_key: String,
    pub share_name: String,
    pub pay_amount: u32,
    pub amount: i32,
    #[serde(with = "standard_format")]
    pub expiration: DateTime<Local>,
    pub expired: u8,
    pub traffic_switch: u8,
    pub traffic_limit_switch: u8,
    pub traffic_limit: u64,
    pub bytes_charge: u64,
    pub preview_count: u32,
    pub download_count: u32,
    pub save_count: u32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayListItem {
    pub last_share_id: i8,
    pub share_list: Option<Vec<PayShareItem>>,
}

pub type SharedDataResponse = ApiResponse<SharedData>;
pub type SharedListDataResponse = ApiResponse<ShareListData>;
pub type PayShareDataResponse = ApiResponse<PayListItem>;
