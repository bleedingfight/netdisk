use actix_files as fs;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use log::{debug, error};
use netdisk_core::netdisk_api::prelude::*;
use netdisk_core::netdisk_auth::basic_env::NetDiskEnv;
use netdisk_core::responses::prelude::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let env = NetDiskEnv::new().map_err(|e| {
        error!("❌ 致命错误：无法初始化 NetDiskEnv：{}", e);
        e // 并将错误返回，导致 main 退出
    })?;

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

    let config_path_data = web::Data::new(env);
    let access_token_data = web::Data::new(access_token);
    HttpServer::new(move || {
        App::new()
            .app_data(config_path_data.clone())
            .app_data(access_token_data.clone())
            .service(echo)
            .service(file_query)
            .service(file_lists_query)
            .route("/access_token", web::post().to(access_token_and_cache))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
