use netdisk_client::{NetdiskClient, NetdiskConfig, FileListQuery};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端配置
    let config = NetdiskConfig {
        client_id: std::env::var("NETDISK_CLIENT_ID").unwrap_or_default(),
        client_secret: std::env::var("NETDISK_CLIENT_SECRET").unwrap_or_default(),
        timeout: Duration::from_secs(30),
        ..Default::default()
    };

    // 创建客户端
    let mut client = NetdiskClient::new(config);

    // 获取访问令牌
    println!("获取访问令牌...");
    match client.get_access_token().await {
        Ok(token_response) => {
            println!("获取令牌成功: {}", token_response.access_token);
        }
        Err(e) => {
            eprintln!("获取令牌失败: {}", e);
            return Ok(());
        }
    }

    // 获取用户信息
    println!("获取用户信息...");
    match client.get_user_info().await {
        Ok(user_info) => {
            if let Some(data) = user_info.data {
                println!("用户昵称: {}", data.nickname);
                println!("已用空间: {} bytes", data.space_used);
            } else {
                println!("用户信息为空");
            }
        }
        Err(e) => {
            eprintln!("获取用户信息失败: {}", e);
        }
    }

    // 获取文件列表
    println!("获取文件列表...");
    let file_list_query = FileListQuery {
        parent_file_id: 0, // 根目录
        limit: 10,
        search_data: None,
        search_mode: None,
        last_file_id: None,
    };

    match client.get_file_list(file_list_query).await {
        Ok(file_list) => {
            if let Some(data) = file_list.data {
                println!("文件数量: {}", data.file_list.len());
                for file in &data.file_list {
                    println!("- {} (ID: {})", file.filename, file.file_id);
                }
            } else {
                println!("文件列表为空");
            }
        }
        Err(e) => {
            eprintln!("获取文件列表失败: {}", e);
        }
    }

    Ok(())
}