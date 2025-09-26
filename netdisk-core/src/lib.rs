use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    client_id: String,
    client_secret: String,
    server: Option<ServerConfig>, // 可选字段
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
            server: None,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            client_id: "123".to_string(),
            client_secret: "123".to_string(),
            server: None,
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // 1. 优先从环境变量读取
    if let (Ok(id), Ok(secret)) = (
        env::var("NETDISK_CLIENT_ID"),
        env::var("NETDISK_CLIENT_SECRET"),
    ) {
        return Ok(Config {
            client_id: id,
            client_secret: secret,
            server: None,
        });
    }

    // 2. ~/.config/netdisk_tool/config.toml
    let home = home::home_dir().expect("Could not find home directory");
    let file = home
        .join(".config")
        .join("netdisk_tools")
        .join("config.toml");

    let mut paths = Vec::new();
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("netdisk_tool/config.toml");
        paths.push(config_dir);
    }

    // 3. 当前目录下 config.toml
    paths.push(PathBuf::from("config.toml"));

    for path in paths {
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("读取配置文件失败 {}: {}", path.display(), e))?;
            let cfg: Config = toml::from_str(&content)
                .map_err(|e| format!("解析 TOML 失败 {}: {}", path.display(), e))?;
            return Ok(cfg);
        }
    }

    // 4. 全部失败
    Err("未找到配置：请设置环境变量 NETDISK_CLIENT_ID/SECRET 或提供 config.toml".into())
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
