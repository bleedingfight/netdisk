use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fs;
use std::path::PathBuf;

/// 授权信息
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    client_id: String,
    client_secret: String,
}

impl AuthConfig {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }
    pub fn new(c_id: String, c_sec: String) -> Self {
        AuthConfig {
            client_id: c_id,
            client_secret: c_sec,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            client_id: "123".to_string(),
            client_secret: "123".to_string(),
        }
    }
}
