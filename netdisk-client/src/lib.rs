//! Netdisk Client Library
//! 
//! A Rust client for the netdisk API using reqwest.

pub mod client;
pub mod error;
pub mod api;

pub use client::{NetdiskClient, NetdiskConfig};
pub use error::NetdiskError;
pub use api::*;

/// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
pub use anyhow::Result;