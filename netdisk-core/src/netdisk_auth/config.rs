use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    client_id: String,
    client_secret: String,
    // server: Option<ServerConfig>, // 可选字段
}

impl Config {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    pub fn new(c_id: String, c_sec: String) -> Self {
        Config {
            client_id: c_id,
            client_secret: c_sec,
            // server: None,
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
            // server: None,
        }
    }
}

// pub fn load_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
//     // 1. 优先从环境变量读取
//     if let (Ok(id), Ok(secret)) = (
//         env::var("NETDISK_CLIENT_ID"),
//         env::var("NETDISK_CLIENT_SECRET"),
//     ) {
//         return Ok(Config {
//             client_id: id,
//             client_secret: secret,
//             // server: None,
//         });
//     }

//     // 2. ~/.config/netdisk_tool/config.toml
//     let home = home::home_dir().expect("Could not find home directory");
//     let file = home
//         .join(".config")
//         .join("netdisk_tools")
//         .join("config.toml");

//     let mut paths = Vec::new();
//     paths.push(file);
//     if let Some(mut config_dir) = dirs::config_dir() {
//         config_dir.push("netdisk_tool/config.toml");
//         paths.push(config_dir);
//     }

//     // 3. 当前目录下 config.toml
//     paths.push(PathBuf::from("config.toml"));

//     for path in paths {
//         if path.exists() {
//             let content = fs::read_to_string(&path)
//                 .map_err(|e| format!("读取配置文件失败 {}: {}", path.display(), e))?;
//             let cfg: Config = toml::from_str(&content)
//                 .map_err(|e| format!("解析 TOML 失败 {}: {}", path.display(), e))?;
//             return Ok(cfg);
//         }
//     }

//     // 4. 全部失败
//     Err("未找到配置：请设置环境变量 NETDISK_CLIENT_ID/SECRET 或提供 config.toml".into())
// }

#[derive(Serialize, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
