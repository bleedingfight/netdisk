use crate::io_basic::read_and_write::*;
use crate::netdisk_auth::basic_env::NetDiskEnv;
use crate::responses::prelude::*;
use actix_web::{get, post, web, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use log::{debug, error, info};
use reqwest;
use std::error::Error;
use std::path::Path;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello 123Pan!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
