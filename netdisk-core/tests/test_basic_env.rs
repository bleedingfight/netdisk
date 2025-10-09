#[cfg(test)]
mod tests {

    use netdisk_core::netdisk_auth::basic_env::NetDiskEnv;

    #[test]
    fn test_basic_env() -> Result<(), Box<dyn std::error::Error>> {
        match NetDiskEnv::new() {
            Ok(env) => {
                println!("🚀 NetDisk 配置已加载！");
                println!("最终配置目录: {}", env.config_dir.display());

                // 示例：获取配置文件路径
                let config_file = env.config_dir.join("settings.json");
                println!("配置文件路径: {}", config_file.display());
            }
            Err(e) => {
                eprintln!("❌ 致命错误：无法初始化 NetDiskEnv：{}", e);
            }
        }

        Ok(())
    }
}
