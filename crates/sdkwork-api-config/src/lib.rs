use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
pub use sdkwork_api_cache_core::CacheBackendKind;
pub use sdkwork_api_secret_core::SecretBackendKind;
use sdkwork_api_storage_core::StorageDialect;
use serde::Deserialize;


mod env_keys;
mod types;
mod loader;
mod http_exposure;
mod standalone_config;
mod config_support;

pub use loader::*;
pub use types::*;

pub(crate) use config_support::*;
pub(crate) use env_keys::*;
pub(crate) use loader::{StandaloneConfigFile, StandaloneConfigWatchEntry};
