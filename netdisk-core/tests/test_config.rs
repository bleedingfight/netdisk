#[cfg(test)]
mod tests {

    use netdisk_core::netdisk_auth::config::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;
    #[test]
    fn test_config_env() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("NETDISK_CLIENT_ID", "test_client_id");
        env::set_var("NETDISK_CLIENT_SECRET", "test_client_secret");
        // let config = load_config()?;
        let config = Config::load()?;
        assert!(!config.client_id().is_empty());
        assert!(!config.client_secret().is_empty());
        env::remove_var("NETDISK_CLIENT_ID");
        env::remove_var("NETDISK_CLIENT_SECRET");

        Ok(())
    }
    #[test]
    fn test_config_file() -> Result<(), Box<dyn std::error::Error>> {
        let home = home::home_dir().expect("Could not find home directory");
        let config_path = home.join(".config").join("netdisk_tools");
        fs::create_dir_all(&config_path)?;

        let tmp_dir = TempDir::new_in(config_path).expect("无法创建临时目录");

        println!("临时目录路径: {:?}", tmp_dir.path());

        // 使用临时目录创建文件
        let file_path = tmp_dir.path().join("config.toml");
        println!("临时文件路径:{:?}", file_path);

        let cfg = Config::new(
            "my_client_id".to_string(),
            "my_client_secret".to_string(),
            Some(PlatformConfig::default()),
        );
        // 序列化为 TOML 字符串
        let toml_str = toml::to_string_pretty(&cfg).expect("序列化为 TOML 失败");

        // 写入文件
        fs::write(&file_path, toml_str)?;
        println!("数据写入临时文件：{:?}", &file_path);

        // let config = load_config()?;
        let config = Config::load()?;
        assert!(config.client_id() == "my_client_id");
        assert!(!config.client_secret().is_empty());
        Ok(())
    }
}
