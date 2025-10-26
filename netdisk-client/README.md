# Netdisk Client

一个使用Rust和reqwest实现的网盘API客户端，基于README中的curl命令实现所有功能。

## 功能特性

- ✅ 认证管理（获取访问令牌）
- ✅ 文件管理（列表、查询、创建、移动、删除）
- ✅ 分享管理（创建分享、分享列表、付费分享）
- ✅ 上传和下载管理
- ✅ 用户信息查询
- ✅ 完整的错误处理
- ✅ 异步支持

## 安装

在`Cargo.toml`中添加依赖：

```toml
[dependencies]
netdisk-client = { path = "./netdisk-client" }
```

## 快速开始

```rust
use netdisk_client::{NetdiskClient, NetdiskConfig};
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
    let token = client.get_access_token().await?;
    println!("Access token: {}", token.access_token);

    // 获取用户信息
    let user_info = client.get_user_info().await?;
    println!("User: {}", user_info.nickname);

    // 获取文件列表
    let file_list = client.get_file_list(netdisk_client::FileListQuery {
        parent_file_id: 0,
        limit: 10,
        search_data: None,
        search_mode: None,
        last_file_id: None,
    }).await?;

    for file in file_list.file_list {
        println!("File: {} (ID: {})", file.filename, file.file_id);
    }

    Ok(())
}
```

## API 功能列表

### 认证 API
- `get_access_token()` - 获取访问令牌

### 文件管理 API
- `get_file_list(query: FileListQuery)` - 获取文件列表
- `get_file_info(file_id: i64)` - 获取单个文件信息
- `get_files_info(file_ids: Vec<u64>)` - 获取多个文件信息
- `create_directory(name: String, parent_id: u64)` - 创建目录
- `move_files(file_ids: Vec<u64>, to_parent_file_id: u64)` - 移动文件
- `move_to_trash(file_ids: Vec<u64>)` - 移动到回收站
- `delete_files(file_ids: Vec<u64>)` - 永久删除文件
- `get_download_info(file_id: i64)` - 获取下载信息

### 分享管理 API
- `get_share_list(limit: u8, last_share_id: Option<i64>)` - 获取分享列表
- `create_share(request: ShareItem)` - 创建分享链接
- `get_payment_list(limit: u8, last_share_id: Option<i64>)` - 获取付费分享列表
- `set_share_info(request: ShareLinkItem)` - 设置分享链接参数
- `create_paid_share(request: PayLinkItem)` - 创建付费分享链接
- `modify_paid_share_info(request: ShareLinkItem)` - 修改付费分享链接

### 用户信息 API
- `get_user_info()` - 获取用户信息

### 上传 API
- `upload_file(request: UploadFileItem)` - 上传文件（创建上传会话）

## 环境变量

设置以下环境变量进行认证：

```bash
export NETDISK_CLIENT_ID="your_client_id"
export NETDISK_CLIENT_SECRET="your_client_secret"
```

## 运行示例

```bash
# 设置环境变量
export NETDISK_CLIENT_ID="your_client_id"
export NETDISK_CLIENT_SECRET="your_client_secret"

# 运行示例
cargo run --example basic_usage
```

## 错误处理

所有API方法都返回`Result<T, NetdiskError>`，支持详细的错误信息：

```rust
match client.get_user_info().await {
    Ok(user_info) => {
        // 处理成功响应
    }
    Err(NetdiskError::ApiError { code, message }) => {
        // 处理API错误
        eprintln!("API Error {}: {}", code, message);
    }
    Err(e) => {
        // 处理其他错误
        eprintln!("Error: {}", e);
    }
}
```

## 许可证

MIT OR Apache-2.0