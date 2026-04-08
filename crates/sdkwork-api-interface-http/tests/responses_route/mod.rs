use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use tower::ServiceExt;

#[path = "../support/mod.rs"]
mod support;

mod billing_and_stream;
mod failover_policy;
mod native_dynamic;
mod retry_policy;
mod route_support;
mod stateful_core;
mod stateless;
mod upstream_fixtures;

use route_support::*;
use upstream_fixtures::*;
