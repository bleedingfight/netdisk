mod tests {
    use netdisk_core::netdisk_auth::config::AccessToken;
    use netdisk_core::{io_basic::read_and_write::*, netdisk_auth::config::AuthConfig};
    use std::path::Path;
    use tempfile::NamedTempFile;
    use tokio;
    #[test]
    fn test_write_toml() {
        let a = AuthConfig::new("123".to_string(), "123".to_string());
        let temp_file = NamedTempFile::new().expect("无法创建临时文件");
        let _ = write_struct_to_toml(&a, temp_file.path());
    }
    #[tokio::test]
    async fn test_async_write_toml() {
        let a = AuthConfig::new("123".to_string(), "123".to_string());
        let temp_file = NamedTempFile::new().expect("无法创建临时文件");
        let _ = async_write_toml(a, temp_file).await;
        println!("异步测试通过！");
    }
    #[tokio::test]
    async fn test_async_read_toml() {
        let file_path = Path::new("/home/liushuai/.config/netdisk/config.toml");
        match async_read_and_deserialize::<_, AccessToken>(&file_path).await {
            Ok(config) => {
                // let current = Utc::now();
                println!("===> current time = {:?}", &config);
            }
            Err(e) => {
                println!("====> message: {}", e);
            }
        }
    }
}
