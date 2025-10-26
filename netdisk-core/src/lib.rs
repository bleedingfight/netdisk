pub mod endpoints;
pub mod io_basic;
pub mod netdisk_api;
pub mod netdisk_auth;
pub mod responses;

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use log::{debug, error};
use netdisk_api::prelude::*;
use netdisk_auth::basic_env::NetDiskEnv;
use responses::prelude::*;

// pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
//     download::configure(cfg);
//     upload::configure(cfg);
// }

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
        .app_data(config_path_data.clone())
        .app_data(access_token_data.clone())
        .service(echo)
        .service(user_info)
        .service(file_search)
        .service(file_upload)
        .service(trash)
        .service(delete)
        .service(move_file)
        .configure(share_config)
        .configure(file_config)
        .route("/access_token", web::post().to(access_token_and_cache))
        .service(fs::Files::new("/", "./static/").index_file("index.html"))
        .route("/hey", web::get().to(manual_hello))
}
