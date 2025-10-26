pub mod endpoints;
pub mod io_basic;
pub mod netdisk_api;
pub mod netdisk_auth;
pub mod responses;

use actix_files as fs;
use actix_web::dev::Service;
use actix_web::{web, App};
use netdisk_api::prelude::*;
use netdisk_auth::basic_env::NetDiskEnv;
use responses::prelude::*;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    netdisk_api::file_api::file_config(cfg);
    netdisk_api::share_file_api::share_config(cfg);
    // netdisk_api::file_move_api::move_config(cfg);
}

pub fn create_app(
    config_path_data: web::Data<NetDiskEnv>,
    access_token_data: web::Data<AccessToken>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap_fn(|req, srv| {
            let method = req.method().clone();
            let path = req.path().to_string();
            let fut = srv.call(req);
            async move {
                println!("📥 收到请求: {} {}", method, path);
                let res = fut.await;
                println!("返回结果: {:?}", &res);
                res
            }
        })
        .app_data(config_path_data.clone())
        .app_data(access_token_data.clone())
        .service(echo)
        .service(user_info)
        .service(file_search)
        .service(file_upload)
        .service(trash)
        .service(delete)
        .service(move_file)
        .configure(configure)
        .route("/access_token", web::post().to(access_token_and_cache))
        .route("/hey", web::get().to(manual_hello))
        .service(fs::Files::new("/static", "./static/").index_file("index.html"))
}
