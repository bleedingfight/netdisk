use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use log::{error, info, warn};
use netdisk_core::netdisk_api::base_api::*;
use netdisk_core::netdisk_auth::config::*;
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init();
//     info!("启动程序");

//     // println!("✅ 成功加载配置");
//     // let config = load_config()?;
//     // println!("client_id = {}", config.client_id());
//     // println!("client_secret 已加载 (不打印以保证安全)");
//     match Config::load() {
//         Ok(config) => println!("配置加载成功: {:?}", config),
//         Err(e) => eprintln!("错误: {}", e),
//     }
//     // if let Some(server) = config.server {
//     //     println!("server = {}:{}", server.host, server.port);
//     // }

//     Ok(())
// }
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(access_token)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
