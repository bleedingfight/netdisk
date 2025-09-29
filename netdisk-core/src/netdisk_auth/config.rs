use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

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
#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_at: u64,
}
impl AccessTokenResponse {
    pub fn new(token: String, expires: u64) -> Self {
        AccessTokenResponse {
            access_token: token,
            expires_at: expires,
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

impl Responder for AccessTokenResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}
