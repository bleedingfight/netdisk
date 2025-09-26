use log::{error, info, warn};
use netdisk_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("启动程序");

    println!("✅ 成功加载配置");
    let config = load_config()?;
    println!("client_id = {}", config.client_id());
    println!("client_secret 已加载 (不打印以保证安全)");

    // if let Some(server) = config.server {
    //     println!("server = {}:{}", server.host, server.port);
    // }

    Ok(())
}
