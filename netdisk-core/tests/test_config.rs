#[cfg(test)]
mod tests {

    use netdisk_core::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;
    #[test]
    fn test_config_env() -> Result<(), Box<dyn std::error::Error>> {
        env::set_var("NETDISK_CLIENT_ID", "test_client_id");
        env::set_var("NETDISK_CLIENT_SECRET", "test_client_secret");
        let config = load_config()?;
        assert!(!config.client_id().is_empty());
        assert!(!config.client_secret().is_empty());
        env::remove_var("NETDISK_CLIENT_ID");
        env::remove_var("NETDISK_CLIENT_SECRET");

        Ok(())
    }
    #[test]
    fn test_config_file() -> Result<(), Box<dyn std::error::Error>> {
        let home = home::home_dir().expect("Could not find home directory");
        let file = home
            .join(".config")
            .join("netdisk_tools")
            .join("config.toml");
        if !file.exists() {
            fs::create_dir_all(&file.parent().unwrap())?
        }
        let cfg = Config::new("my_client_id".to_string(), "my_client_secret".to_string());

        // 序列化为 TOML 字符串
        let toml_str = toml::to_string_pretty(&cfg).expect("序列化为 TOML 失败");

        // 写入文件
        fs::write(file, toml_str)?;

        let config = load_config()?;
        println!("==========> test = {:?}", config.client_id());
        assert!(config.client_id() == "my_client_id");
        assert!(!config.client_secret().is_empty());
        Ok(())
    }
}
