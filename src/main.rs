use actix_web::web;
use actix_web::HttpServer;
use log::{debug, error};
use netdisk_core::create_app;
use netdisk_core::netdisk_api::prelude::*;
use netdisk_core::netdisk_auth::basic_env::NetDiskEnv;
use netdisk_core::responses::prelude::*;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let env = match NetDiskEnv::new() {
        Ok(env) => env,
        Err(e) => {
            error!("❌ 致命错误：无法初始化 NetDiskEnv：{}", e);
            return Ok(());
        }
    };
    let file_path = env.config_dir.join("config.toml");
    let mut access_token: AccessToken = AccessToken::default();
    match get_access_token_from_cache(&file_path).await {
        Ok(token) => {
            access_token = token;
        }
        Err(_) => {
            debug!("Error to message");
        }
    }

    // 注入全局数据
    let config_path_data = web::Data::new(env);
    let access_token_data = web::Data::new(access_token);

    HttpServer::new(move || create_app(config_path_data.clone(), access_token_data.clone()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
